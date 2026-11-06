use chrono::{DateTime, Duration, Utc};
use common::OrganizationId;
use uuid::Uuid;

use super::secret::SealedSecret;

pub fn oauth_state_lifetime() -> Duration {
    Duration::minutes(10)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OauthState {
    pub org_id: OrganizationId,
    pub credential_id: Uuid,
    pub member_id: Uuid,
    pub redirect_uri: String,
    pub return_to: String,
    pub payload: Option<SealedSecret>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl OauthState {
    pub fn new(
        org_id: OrganizationId,
        credential_id: Uuid,
        member_id: Uuid,
        redirect_uri: String,
        return_to: String,
        payload: Option<SealedSecret>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            org_id,
            credential_id,
            member_id,
            redirect_uri,
            return_to,
            payload,
            created_at: now,
            expires_at: now + oauth_state_lifetime(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OauthCallback {
    pub state: String,
    pub code: Option<String>,
    pub provider_refused: bool,
    pub oauth_token: Option<String>,
    pub oauth_verifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OauthOutcome {
    pub return_to: String,
    pub connected: bool,
}

pub const MAX_ADDRESS_BYTES: usize = 2048;

pub const DEVELOPMENT_ORIGIN: &str = "http://localhost:3000";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectRefusal {
    Invalid,
    NotConfigured,
}

pub fn expected_redirect_uri(public_api_url: &str) -> String {
    format!(
        "{}/oauth/callback",
        public_api_url.trim().trim_end_matches('/')
    )
}

fn is_loopback_host(candidate: &str) -> bool {
    let Some((_, rest)) = candidate.split_once("://") else {
        return false;
    };
    let authority = rest.split('/').next().unwrap_or_default();
    let host = match authority.strip_prefix('[') {
        Some(inner) => inner.split(']').next().unwrap_or_default(),
        None => authority.split(':').next().unwrap_or_default(),
    };
    matches!(
        host.to_ascii_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "::1"
    )
}

pub fn check_redirect_uri(
    candidate: &str,
    public_api_url: Option<&str>,
) -> Result<(), RedirectRefusal> {
    if !is_valid_redirect_uri(candidate) {
        return Err(RedirectRefusal::Invalid);
    }
    match public_api_url.map(str::trim).filter(|url| !url.is_empty()) {
        Some(public) if candidate == expected_redirect_uri(public) => Ok(()),
        Some(_) => Err(RedirectRefusal::Invalid),
        None if is_loopback_host(candidate) => Ok(()),
        None => Err(RedirectRefusal::NotConfigured),
    }
}

pub fn return_origins(public_api_url: Option<&str>, allowed_origins: &[String]) -> Vec<String> {
    let mut origins = Vec::with_capacity(allowed_origins.len() + 1);
    if public_api_url.map(str::trim).is_none_or(str::is_empty) {
        origins.push(DEVELOPMENT_ORIGIN.to_owned());
    }
    origins.extend(allowed_origins.iter().cloned());
    origins
}

pub fn is_valid_redirect_uri(candidate: &str) -> bool {
    if candidate.len() > MAX_ADDRESS_BYTES {
        return false;
    }
    let Some((scheme_end, rest)) = candidate.split_once("://") else {
        return false;
    };
    if !matches!(scheme_end, "http" | "https") || rest.is_empty() {
        return false;
    }
    if candidate
        .chars()
        .any(|c| c.is_whitespace() || c.is_control())
    {
        return false;
    }
    if candidate.contains('?') || candidate.contains('#') {
        return false;
    }
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    !authority.is_empty()
        && !authority.contains('@')
        && path.trim_end_matches('/').ends_with("oauth/callback")
}

pub fn is_valid_return_to(candidate: &str, allowed_origins: &[String]) -> bool {
    if candidate.len() > MAX_ADDRESS_BYTES {
        return false;
    }
    if candidate
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return false;
    }
    if candidate.contains('#') {
        return false;
    }
    if let Some(path) = candidate.strip_prefix('/') {
        return !path.starts_with('/');
    }
    let Some((scheme, rest)) = candidate.split_once("://") else {
        return false;
    };
    if !matches!(scheme, "http" | "https") {
        return false;
    }
    let authority = rest.split(['/', '?']).next().unwrap_or_default();
    if authority.is_empty() || authority.contains('@') {
        return false;
    }
    let origin = format!("{scheme}://{authority}");
    allowed_origins
        .iter()
        .any(|allowed| allowed.trim_end_matches('/').eq_ignore_ascii_case(&origin))
}

pub fn with_outcome(return_to: &str, connected: bool) -> String {
    let separator = if return_to.contains('?') { '&' } else { '?' };
    let outcome = if connected { "ok" } else { "error" };
    format!("{return_to}{separator}oauth={outcome}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed() -> Vec<String> {
        vec![
            "http://localhost:3000".to_owned(),
            "https://app.mestier.fr/".to_owned(),
        ]
    }

    #[test]
    fn a_state_lives_ten_minutes_from_its_creation() {
        let now = Utc::now();

        let state = OauthState::new(
            OrganizationId(Uuid::from_u128(1)),
            Uuid::from_u128(2),
            Uuid::from_u128(3),
            "https://api.mestier.fr/oauth/callback".to_owned(),
            "/".to_owned(),
            None,
            now,
        );

        assert_eq!(state.expires_at - state.created_at, Duration::minutes(10));
        assert_eq!(state.created_at, now);
    }

    #[test]
    fn a_redirect_uri_is_the_callback_of_an_http_origin_without_query_fragment_or_login() {
        for good in [
            "https://api.mestier.fr/oauth/callback",
            "http://localhost:3456/oauth/callback",
            "https://api.mestier.fr/prefix/oauth/callback",
            "https://api.mestier.fr/oauth/callback/",
        ] {
            assert!(is_valid_redirect_uri(good), "{good}");
        }
        for bad in [
            "",
            "/oauth/callback",
            "ftp://api.mestier.fr/oauth/callback",
            "https://api.mestier.fr/oauth/callback?x=1",
            "https://api.mestier.fr/oauth/callback#frag",
            "https://user@api.mestier.fr/oauth/callback",
            "https://api.mestier.fr/elsewhere",
            "https://api.mestier.fr",
            "https:///oauth/callback",
            "https://api.mestier.fr/oauth/callback x",
            "javascript:alert(1)//oauth/callback",
        ] {
            assert!(!is_valid_redirect_uri(bad), "{bad}");
        }
    }

    #[test]
    fn an_address_longer_than_two_kibibytes_is_refused_and_a_nul_too() {
        let long_path = "a".repeat(MAX_ADDRESS_BYTES);

        assert!(!is_valid_redirect_uri(&format!(
            "https://api.mestier.fr/{long_path}/oauth/callback"
        )));
        assert!(!is_valid_return_to(&format!("/{long_path}"), &allowed()));
        assert!(is_valid_return_to(
            &format!("/{}", "a".repeat(MAX_ADDRESS_BYTES - 1)),
            &allowed()
        ));
        assert!(!is_valid_redirect_uri(
            "https://api.mestier.fr/\0/oauth/callback"
        ));
        assert!(!is_valid_return_to("/ok\0", &allowed()));
    }

    #[test]
    fn a_configured_public_url_is_the_only_redirect_uri_accepted() {
        let public = Some("https://api.mestier.fr/");

        assert_eq!(
            check_redirect_uri("https://api.mestier.fr/oauth/callback", public),
            Ok(())
        );
        for bad in [
            "https://evil.example/oauth/callback",
            "https://api.mestier.fr.evil.example/oauth/callback",
            "http://api.mestier.fr/oauth/callback",
            "https://api.mestier.fr/prefix/oauth/callback",
            "https://api.mestier.fr/oauth/callback/",
            "http://localhost:3456/oauth/callback",
            "https://api.mestier.fr/elsewhere",
        ] {
            assert_eq!(
                check_redirect_uri(bad, public),
                Err(RedirectRefusal::Invalid),
                "{bad}"
            );
        }
        assert_eq!(
            expected_redirect_uri("https://api.mestier.fr///"),
            "https://api.mestier.fr/oauth/callback"
        );
    }

    #[test]
    fn without_a_public_url_only_a_loopback_host_is_accepted_and_anything_else_is_not_configured() {
        for good in [
            "http://localhost:3456/oauth/callback",
            "http://127.0.0.1:3456/oauth/callback",
            "http://[::1]:3456/oauth/callback",
            "http://LOCALHOST/oauth/callback",
        ] {
            assert_eq!(check_redirect_uri(good, None), Ok(()), "{good}");
            assert_eq!(check_redirect_uri(good, Some("  ")), Ok(()), "{good}");
        }
        for unconfigured in [
            "https://api.mestier.fr/oauth/callback",
            "http://localhost.evil.example/oauth/callback",
            "http://127.0.0.1.evil.example/oauth/callback",
            "http://10.0.0.5/oauth/callback",
        ] {
            assert_eq!(
                check_redirect_uri(unconfigured, None),
                Err(RedirectRefusal::NotConfigured),
                "{unconfigured}"
            );
        }
        assert_eq!(
            check_redirect_uri("not an address", None),
            Err(RedirectRefusal::Invalid)
        );
    }

    #[test]
    fn the_development_origin_only_exists_while_no_public_url_is_configured() {
        let allowed = vec!["https://app.mestier.fr".to_owned()];

        assert_eq!(
            return_origins(None, &allowed),
            vec![
                DEVELOPMENT_ORIGIN.to_owned(),
                "https://app.mestier.fr".to_owned()
            ]
        );
        assert_eq!(
            return_origins(Some("https://api.mestier.fr"), &allowed),
            allowed
        );
        assert!(!is_valid_return_to(
            "http://localhost:3000/o/x",
            &return_origins(Some("https://api.mestier.fr"), &allowed)
        ));
        assert!(is_valid_return_to(
            "http://localhost:3000/o/x",
            &return_origins(None, &allowed)
        ));
    }

    #[test]
    fn a_return_to_is_a_local_path_or_an_allowed_origin() {
        for good in [
            "/o/atelier/automatisation/credentials",
            "/automatisation/credentials?tab=1",
            "http://localhost:3000/o/atelier/automatisation/credentials",
            "https://app.mestier.fr/o/x/credentials",
            "https://APP.mestier.fr",
        ] {
            assert!(is_valid_return_to(good, &allowed()), "{good}");
        }
        for bad in [
            "",
            "//evil.example/path",
            "/\\evil.example",
            "https://evil.example/o/x",
            "https://app.mestier.fr.evil.example/",
            "https://user@app.mestier.fr/",
            "http://app.mestier.fr/",
            "javascript:alert(1)",
            "/ok#fragment",
            "relative/path",
            "https://app.mestier.fr/ with space",
        ] {
            assert!(!is_valid_return_to(bad, &allowed()), "{bad}");
        }
    }

    #[test]
    fn the_outcome_is_appended_to_the_return_address() {
        assert_eq!(with_outcome("/credentials", true), "/credentials?oauth=ok");
        assert_eq!(
            with_outcome("/credentials?tab=1", false),
            "/credentials?tab=1&oauth=error"
        );
    }
}
