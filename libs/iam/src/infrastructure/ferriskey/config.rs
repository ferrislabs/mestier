/// Static configuration for [`FerriskeyIamProvider`](super::FerriskeyIamProvider).
///
/// Loaded once at boot from the API's `args` crate / environment. The
/// `client_secret` is opaque on purpose — never logged, never displayed.
#[derive(Clone)]
pub struct FerriskeyConfig {
    /// Base URL of the Ferriskey realm (e.g. `https://iam.example.com/realms/mestier`).
    /// Trailing slash is normalized away.
    pub issuer: String,

    /// Service-account client identifier used by the Mestier API to talk to the
    /// Ferriskey admin API.
    pub client_id: String,

    /// Service-account client secret. Held in memory for the lifetime of the
    /// process; not derived from per-request state.
    pub client_secret: String,
}

impl FerriskeyConfig {
    pub fn new(
        issuer: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Self {
        let issuer: String = issuer.into();
        Self {
            issuer: issuer.trim_end_matches('/').to_owned(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        }
    }
}

impl std::fmt::Debug for FerriskeyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FerriskeyConfig")
            .field("issuer", &self.issuer)
            .field("client_id", &self.client_id)
            .field("client_secret", &"***")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_trailing_slash_from_issuer() {
        let cfg = FerriskeyConfig::new("https://iam.example.com/realms/mestier/", "id", "secret");
        assert_eq!(cfg.issuer, "https://iam.example.com/realms/mestier");
    }

    #[test]
    fn debug_redacts_client_secret() {
        let cfg = FerriskeyConfig::new("https://iam.example.com", "id", "super-secret");
        let dbg = format!("{cfg:?}");
        assert!(!dbg.contains("super-secret"));
        assert!(dbg.contains("***"));
    }
}
