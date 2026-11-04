use axum::http::{HeaderMap, header::AUTHORIZATION};
use handlers::{ApiError, AppState};
use mestier_core::OrganizationId;

pub async fn require_intake_key(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<OrganizationId, ApiError> {
    let plaintext = bearer_key(headers).ok_or(ApiError::Unauthorized)?;

    state
        .usecase
        .authenticate_support_api_key(plaintext)
        .await?
        .ok_or(ApiError::Unauthorized)
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
}
