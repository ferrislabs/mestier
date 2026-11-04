use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, header::AUTHORIZATION, request::Parts},
};
use handlers::{ApiError, AppState};
use mestier_core::{OrganizationId, PresentedApiKey};

pub struct IntakeOrganization(pub OrganizationId);

impl FromRequestParts<AppState> for IntakeOrganization {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        require_intake_key(state, &parts.headers).await.map(Self)
    }
}

pub async fn require_intake_key(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<OrganizationId, ApiError> {
    let plaintext = presented_key(headers).ok_or(ApiError::Unauthorized)?;

    state
        .usecase
        .authenticate_support_api_key(plaintext)
        .await?
        .ok_or(ApiError::Unauthorized)
}

fn presented_key(headers: &HeaderMap) -> Option<&str> {
    bearer_key(headers).filter(|key| PresentedApiKey::parse(key).is_some())
}

fn bearer_key(headers: &HeaderMap) -> Option<&str> {
    let mut values = headers.get_all(AUTHORIZATION).iter();
    let value = values.next()?;
    if values.next().is_some() {
        return None;
    }

    let (scheme, key) = value.to_str().ok()?.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") || key.is_empty() || key.contains(' ') {
        return None;
    }

    Some(key)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    const KEY: &str = "0a1b2c3d.00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn headers(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(value).unwrap());
        headers
    }

    #[test]
    fn a_bearer_header_yields_the_key() {
        assert_eq!(bearer_key(&headers(&format!("Bearer {KEY}"))), Some(KEY));
    }

    #[test]
    fn the_scheme_is_case_insensitive() {
        for scheme in ["bearer", "BEARER", "BeArEr"] {
            assert_eq!(
                bearer_key(&headers(&format!("{scheme} {KEY}"))),
                Some(KEY),
                "{scheme}"
            );
        }
    }

    #[test]
    fn no_header_yields_nothing() {
        assert_eq!(bearer_key(&HeaderMap::new()), None);
    }

    #[test]
    fn another_scheme_yields_nothing() {
        for value in [
            format!("Basic {KEY}"),
            format!("Token {KEY}"),
            format!("Bearer{KEY}"),
            format!("Bearers {KEY}"),
            KEY.to_owned(),
        ] {
            assert_eq!(bearer_key(&headers(&value)), None, "{value}");
        }
    }

    #[test]
    fn anything_but_exactly_one_space_yields_nothing() {
        for value in [
            format!("Bearer  {KEY}"),
            format!("Bearer\t{KEY}"),
            format!("Bearer {KEY} "),
            format!("Bearer {KEY} extra"),
            format!(" Bearer {KEY}"),
            "Bearer ".to_owned(),
            "Bearer".to_owned(),
            String::new(),
        ] {
            assert_eq!(bearer_key(&headers(&value)), None, "{value:?}");
        }
    }

    #[test]
    fn two_authorization_headers_yield_nothing() {
        let mut headers = headers(&format!("Bearer {KEY}"));
        headers.append(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {KEY}")).unwrap(),
        );

        assert_eq!(bearer_key(&headers), None);
    }

    #[test]
    fn a_header_that_is_not_text_yields_nothing() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_bytes(b"Bearer \xff\xfe").unwrap(),
        );

        assert_eq!(bearer_key(&headers), None);
    }

    fn refused_shapes() -> Vec<String> {
        let secret = &KEY[9..];
        vec![
            "Bearer x".to_owned(),
            "Bearer ".to_owned(),
            "Bearer".to_owned(),
            String::new(),
            format!("Basic {KEY}"),
            format!("Token {KEY}"),
            format!("Bearer{KEY}"),
            format!("Bearers {KEY}"),
            KEY.to_owned(),
            format!("Bearer  {KEY}"),
            format!("Bearer\t{KEY}"),
            format!("Bearer {KEY} "),
            format!("Bearer {KEY} extra"),
            format!(" Bearer {KEY}"),
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.c2lnbmF0dXJl".to_owned(),
            format!("Bearer 0a1b2c3d.{secret}.{secret}"),
            format!("Bearer {}", KEY.to_uppercase()),
            format!("Bearer 0A1B2C3D.{secret}"),
            format!("Bearer 0a1b2c3d.{}", secret.to_uppercase()),
            format!("Bearer 0a1b2c3.{secret}"),
            format!("Bearer 0a1b2c3d0.{secret}"),
            format!("Bearer 0a1b2c3d.{}", &secret[..63]),
            format!("Bearer 0a1b2c3d.{secret}0"),
            format!("Bearer 0a1b2c3d{secret}"),
            format!("Bearer .{secret}"),
            "Bearer 0a1b2c3d.".to_owned(),
            format!("Bearer 0a1b2c3g.{secret}"),
            format!("Bearer {}", "a".repeat(10_000)),
        ]
    }

    #[test]
    fn a_well_formed_key_is_presented() {
        assert_eq!(presented_key(&headers(&format!("Bearer {KEY}"))), Some(KEY));
        assert_eq!(presented_key(&headers(&format!("bearer {KEY}"))), Some(KEY));
    }

    #[test]
    fn a_header_that_is_not_a_well_formed_key_is_not_presented() {
        for value in refused_shapes() {
            assert_eq!(presented_key(&headers(&value)), None, "{value:?}");
        }
    }

    #[test]
    fn no_header_is_not_presented() {
        assert_eq!(presented_key(&HeaderMap::new()), None);
    }

    #[test]
    fn two_authorization_headers_are_not_presented() {
        let mut headers = headers(&format!("Bearer {KEY}"));
        headers.append(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {KEY}")).unwrap(),
        );

        assert_eq!(presented_key(&headers), None);
    }

    #[test]
    fn a_header_that_is_not_text_is_not_presented() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_bytes(b"Bearer \xff\xfe").unwrap(),
        );

        assert_eq!(presented_key(&headers), None);
    }
}
