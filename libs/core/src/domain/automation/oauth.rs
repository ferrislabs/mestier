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

pub fn is_valid_redirect_uri(candidate: &str) -> bool {
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
