use std::sync::Arc;

use reqwest::Client;
use tokio::sync::RwLock;

use crate::{
    IamCreateOrganization, IamCreateRole, IamCreateUser, IamError, IamOrgId, IamOrganization,
    IamProvider, IamRole, IamRoleId, IamUpdateOrganization, IamUpdateRole, IamUpdateUser, IamUser,
    IamUserId, infrastructure::ferriskey::config::FerriskeyConfig,
};

/// Cached service-account access token.
///
/// Refreshed lazily by [`FerriskeyIamProvider`] before any admin call. Stored
/// behind a [`RwLock`] so concurrent callers can read the cached token without
/// contention; only refreshes acquire the write lock.
#[derive(Default)]
struct TokenCache {
    /// Filled in once `refresh()` lands. Kept private so the only way to
    /// observe it is through the provider's own auth helpers.
    _bearer: Option<String>,
    /// Absolute deadline (Unix seconds) at which `_bearer` becomes invalid.
    /// Refreshes happen a few seconds early to absorb clock skew.
    _expires_at: Option<i64>,
}

/// Adapter implementing [`IamProvider`] against a Ferriskey realm.
///
/// Holds an HTTP client, the config, and a cached service-account token. The
/// trait methods are stubbed for now (`IamError::Internal("not yet
/// implemented")`) — this commit only scaffolds the type so wiring work in
/// other crates can proceed in parallel.
#[derive(Clone)]
pub struct FerriskeyIamProvider {
    config: FerriskeyConfig,
    #[allow(unused)]
    http: Client,
    #[allow(unused)]
    token: Arc<RwLock<TokenCache>>,
}

impl FerriskeyIamProvider {
    pub fn new(config: FerriskeyConfig) -> Self {
        Self {
            config,
            http: Client::new(),
            token: Arc::new(RwLock::new(TokenCache::default())),
        }
    }

    /// Build the provider with an externally-supplied [`reqwest::Client`].
    /// Useful when the API wants to share connection pools / middleware
    /// (timeouts, tracing, retries) across all outbound HTTP calls.
    pub fn with_http_client(config: FerriskeyConfig, http: Client) -> Self {
        Self {
            config,
            http,
            token: Arc::new(RwLock::new(TokenCache::default())),
        }
    }

    pub fn config(&self) -> &FerriskeyConfig {
        &self.config
    }
}

const NOT_IMPLEMENTED: &str = "FerriskeyIamProvider: not yet implemented";

impl IamProvider for FerriskeyIamProvider {
    async fn create_user(&self, _command: IamCreateUser) -> Result<IamUser, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn update_user(
        &self,
        _id: &IamUserId,
        _command: IamUpdateUser,
    ) -> Result<IamUser, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn delete_user(&self, _id: &IamUserId) -> Result<(), IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn find_user(&self, _id: &IamUserId) -> Result<Option<IamUser>, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn find_user_by_email(&self, _email: &str) -> Result<Option<IamUser>, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn create_organization(
        &self,
        _command: IamCreateOrganization,
    ) -> Result<IamOrganization, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn update_organization(
        &self,
        _id: &IamOrgId,
        _command: IamUpdateOrganization,
    ) -> Result<IamOrganization, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn delete_organization(&self, _id: &IamOrgId) -> Result<(), IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn find_organization(&self, _id: &IamOrgId) -> Result<Option<IamOrganization>, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn add_user_to_organization(
        &self,
        _user: &IamUserId,
        _organization: &IamOrgId,
    ) -> Result<(), IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn remove_user_from_organization(
        &self,
        _user: &IamUserId,
        _organization: &IamOrgId,
    ) -> Result<(), IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn create_role(
        &self,
        _organization: &IamOrgId,
        _command: IamCreateRole,
    ) -> Result<IamRole, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn update_role(
        &self,
        _id: &IamRoleId,
        _command: IamUpdateRole,
    ) -> Result<IamRole, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn delete_role(&self, _id: &IamRoleId) -> Result<(), IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn list_roles(&self, _organization: &IamOrgId) -> Result<Vec<IamRole>, IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn assign_role(&self, _user: &IamUserId, _role: &IamRoleId) -> Result<(), IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }

    async fn unassign_role(&self, _user: &IamUserId, _role: &IamRoleId) -> Result<(), IamError> {
        Err(IamError::Internal(NOT_IMPLEMENTED.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider() -> FerriskeyIamProvider {
        FerriskeyIamProvider::new(FerriskeyConfig::new(
            "https://iam.example.com/realms/mestier",
            "mestier-api",
            "secret",
        ))
    }

    #[tokio::test]
    async fn stub_returns_internal_error() {
        let err = provider()
            .find_user_by_email("alice@example.com")
            .await
            .unwrap_err();

        assert!(matches!(err, IamError::Internal(_)));
    }

    #[test]
    fn config_accessor_returns_normalized_issuer() {
        let p = provider();
        assert_eq!(p.config().issuer, "https://iam.example.com/realms/mestier");
    }
}
