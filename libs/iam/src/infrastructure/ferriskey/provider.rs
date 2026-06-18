use std::sync::Arc;

use reqwest::Client;
use serde::Deserialize;
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
    bearer: Option<String>,
    expires_at: Option<i64>,
}

/// FerrisKey user DTO.  Field names are snake_case as FerrisKey uses them
/// natively — no camelCase rename needed.
#[derive(Deserialize)]
struct FerriskeyUser {
    id: String,
    email: String,
    username: String,
    #[serde(default)]
    firstname: Option<String>,
    #[serde(default)]
    lastname: Option<String>,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    email_verified: bool,
}

impl FerriskeyUser {
    /// Join `firstname` and `lastname` into a display name, trimming blanks.
    /// Returns `None` when both are absent or empty.
    fn display_name(&self) -> Option<String> {
        let parts: Vec<&str> = [self.firstname.as_deref(), self.lastname.as_deref()]
            .into_iter()
            .flatten()
            .filter(|s| !s.trim().is_empty())
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" "))
        }
    }

    fn into_iam_user(self) -> IamUser {
        let name = self.display_name();
        IamUser {
            id: IamUserId(self.id),
            email: self.email,
            username: self.username,
            name,
            email_verified: self.email_verified,
            enabled: self.enabled,
        }
    }
}

/// Generic response envelope used by FerrisKey for single-resource responses
/// (`{ "data": <T> }`) and list responses (`{ "data": [<T>, ...] }`).
#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

/// Adapter implementing [`IamProvider`] against a Ferriskey realm.
///
/// Holds an HTTP client, the config, and a cached service-account token.
#[derive(Clone)]
pub struct FerriskeyIamProvider {
    config: FerriskeyConfig,
    http: Client,
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

    /// Derives the OIDC token endpoint from the configured issuer.
    fn token_url(&self) -> String {
        format!("{}/protocol/openid-connect/token", self.config.issuer)
    }

    /// Root of the FerrisKey users API.  The issuer already is the realm base
    /// (e.g. `https://iam.example.com/realms/mestier`), so the users
    /// collection is simply `{issuer}/users`.
    fn users_url(&self) -> String {
        format!("{}/users", self.config.issuer)
    }

    /// URL for a specific user by id.
    fn user_url(&self, id: &str) -> String {
        format!("{}/users/{}", self.config.issuer, id)
    }

    /// Returns a valid bearer token, refreshing it when absent or within 10
    /// seconds of expiry.
    async fn ensure_token(&self) -> Result<String, IamError> {
        // Fast path: read lock — token is fresh.
        {
            let cache = self.token.read().await;
            if let (Some(bearer), Some(exp)) = (&cache.bearer, cache.expires_at) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                if exp - now > 10 {
                    return Ok(bearer.clone());
                }
            }
        }

        // Slow path: acquire write lock and refresh.
        let mut cache = self.token.write().await;

        // Double-check after acquiring write lock.
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            if let (Some(bearer), Some(exp)) = (&cache.bearer, cache.expires_at)
                && exp - now > 10
            {
                return Ok(bearer.clone());
            }
        }

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
        ];

        let resp = self
            .http
            .post(self.token_url())
            .form(&params)
            .send()
            .await
            .map_err(|e| IamError::Unavailable(e.to_string()))?;

        let status = resp.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(IamError::Unauthorized);
        }
        if status.is_server_error() {
            let msg = resp.text().await.unwrap_or_default();
            return Err(IamError::Unavailable(format!(
                "token endpoint returned {status}: {msg}"
            )));
        }
        if !status.is_success() {
            let msg = resp.text().await.unwrap_or_default();
            return Err(IamError::Internal(format!(
                "unexpected token response {status}: {msg}"
            )));
        }

        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
            expires_in: i64,
        }

        let body: TokenResponse = resp
            .json()
            .await
            .map_err(|e| IamError::Internal(format!("failed to parse token response: {e}")))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        cache.bearer = Some(body.access_token.clone());
        cache.expires_at = Some(now + body.expires_in);

        Ok(body.access_token)
    }

    /// Maps an HTTP status code and response body to an [`IamError`].
    fn map_status_to_iam_error(status: reqwest::StatusCode, body: &str) -> IamError {
        match status.as_u16() {
            401 => IamError::Unauthorized,
            403 => IamError::Forbidden,
            404 => IamError::NotFound,
            409 => IamError::Conflict(body.to_owned()),
            400 | 422 => IamError::InvalidInput(body.to_owned()),
            s if s >= 500 => IamError::Unavailable(format!("IAM returned {s}: {body}")),
            s => IamError::Internal(format!("unexpected IAM response {s}: {body}")),
        }
    }
}

const NOT_IMPLEMENTED: &str = "FerriskeyIamProvider: not yet implemented";

impl IamProvider for FerriskeyIamProvider {
    async fn create_user(&self, command: IamCreateUser) -> Result<IamUser, IamError> {
        let token = self.ensure_token().await?;

        // `send_invite_email` is not supported by FerrisKey at create time;
        // the field is intentionally ignored per the IamCreateUser contract.
        let mut payload = serde_json::json!({
            "email": command.email,
            "username": command.username,
            "email_verified": false,
        });
        if let Some(name) = &command.name {
            payload["firstname"] = serde_json::Value::String(name.clone());
        }

        let resp = self
            .http
            .post(self.users_url())
            .bearer_auth(&token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| IamError::Unavailable(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Self::map_status_to_iam_error(status, &body));
        }

        // FerrisKey returns the created user in the response body wrapped in
        // `{ "data": <user> }` — no Location header, id is data.id.
        let envelope: Envelope<FerriskeyUser> = resp.json().await.map_err(|e| {
            IamError::Internal(format!("failed to parse create_user response: {e}"))
        })?;

        Ok(envelope.data.into_iam_user())
    }

    async fn update_user(
        &self,
        id: &IamUserId,
        command: IamUpdateUser,
    ) -> Result<IamUser, IamError> {
        let token = self.ensure_token().await?;

        let mut payload = serde_json::Map::new();
        if let Some(email) = command.email {
            payload.insert("email".into(), serde_json::Value::String(email));
        }
        if let Some(username) = command.username {
            payload.insert("username".into(), serde_json::Value::String(username));
        }
        if let Some(name) = command.name {
            payload.insert("firstname".into(), serde_json::Value::String(name));
        }
        if let Some(enabled) = command.enabled {
            payload.insert("enabled".into(), serde_json::Value::Bool(enabled));
        }

        let url = self.user_url(&id.0);

        let resp = self
            .http
            .put(&url)
            .bearer_auth(&token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| IamError::Unavailable(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Self::map_status_to_iam_error(status, &body));
        }

        // FerrisKey returns the updated user wrapped in `{ "data": <user> }`.
        let envelope: Envelope<FerriskeyUser> = resp.json().await.map_err(|e| {
            IamError::Internal(format!("failed to parse update_user response: {e}"))
        })?;

        Ok(envelope.data.into_iam_user())
    }

    async fn delete_user(&self, id: &IamUserId) -> Result<(), IamError> {
        let token = self.ensure_token().await?;
        let url = self.user_url(&id.0);

        let resp = self
            .http
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| IamError::Unavailable(e.to_string()))?;

        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Self::map_status_to_iam_error(status, &body))
    }

    async fn find_user(&self, id: &IamUserId) -> Result<Option<IamUser>, IamError> {
        let token = self.ensure_token().await?;
        let url = self.user_url(&id.0);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| IamError::Unavailable(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Self::map_status_to_iam_error(status, &body));
        }

        let envelope: Envelope<FerriskeyUser> = resp
            .json()
            .await
            .map_err(|e| IamError::Internal(format!("failed to parse user: {e}")))?;

        Ok(Some(envelope.data.into_iam_user()))
    }

    async fn find_user_by_email(&self, email: &str) -> Result<Option<IamUser>, IamError> {
        // FerrisKey's list endpoint does not support server-side email filtering;
        // we fetch the full list and filter client-side.  A server-side filter
        // query parameter may be added in a future FerrisKey release.
        let token = self.ensure_token().await?;
        let url = self.users_url();

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| IamError::Unavailable(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Self::map_status_to_iam_error(status, &body));
        }

        let envelope: Envelope<Vec<FerriskeyUser>> = resp
            .json()
            .await
            .map_err(|e| IamError::Internal(format!("failed to parse users list: {e}")))?;

        Ok(envelope
            .data
            .into_iter()
            .find(|u| u.email == email)
            .map(FerriskeyUser::into_iam_user))
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

    fn provider_at(addr: std::net::SocketAddr) -> FerriskeyIamProvider {
        FerriskeyIamProvider::new(FerriskeyConfig::new(
            format!("http://{}/realms/test", addr),
            "client",
            "secret",
        ))
    }

    fn serve_responses(responses: Vec<(&'static str, String)>) -> std::net::SocketAddr {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            for (status, body) in responses {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = [0u8; 8192];
                    let _ = stream.read(&mut buf);
                    let resp = format!(
                        "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(resp.as_bytes());
                }
            }
        });
        addr
    }

    // --- URL helpers --------------------------------------------------------

    #[test]
    fn config_accessor_returns_normalized_issuer() {
        let p = FerriskeyIamProvider::new(FerriskeyConfig::new(
            "https://iam.example.com/realms/mestier",
            "mestier-api",
            "secret",
        ));
        assert_eq!(p.config().issuer, "https://iam.example.com/realms/mestier");
    }

    #[test]
    fn users_url_is_issuer_slash_users() {
        let p = FerriskeyIamProvider::new(FerriskeyConfig::new(
            "https://iam.example.com/realms/mestier",
            "id",
            "secret",
        ));
        assert_eq!(
            p.users_url(),
            "https://iam.example.com/realms/mestier/users"
        );
    }

    #[test]
    fn user_url_appends_id() {
        let p = FerriskeyIamProvider::new(FerriskeyConfig::new(
            "https://iam.example.com/realms/mestier",
            "id",
            "secret",
        ));
        assert_eq!(
            p.user_url("abc-123"),
            "https://iam.example.com/realms/mestier/users/abc-123"
        );
    }

    // --- ensure_token -------------------------------------------------------

    #[tokio::test]
    async fn ensure_token_fetches_and_caches_bearer() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = std::io::Read::read(&mut stream, &mut buf);
                let body = r#"{"access_token":"test-bearer","expires_in":300}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
            }
        });

        let provider = provider_at(addr);

        let token = provider.ensure_token().await.unwrap();
        assert_eq!(token, "test-bearer");

        // Second call must NOT hit the network (server is already done).
        let token2 = provider.ensure_token().await.unwrap();
        assert_eq!(token2, "test-bearer");
    }

    #[tokio::test]
    async fn ensure_token_returns_unavailable_on_5xx() {
        let addr = serve_responses(vec![(
            "500 Internal Server Error",
            r#"{"error":"server_error"}"#.into(),
        )]);

        let err = provider_at(addr).ensure_token().await.unwrap_err();
        assert!(matches!(err, IamError::Unavailable(_)));
    }

    #[tokio::test]
    async fn ensure_token_returns_unauthorized_on_401() {
        let addr = serve_responses(vec![(
            "401 Unauthorized",
            r#"{"error":"unauthorized_client"}"#.into(),
        )]);

        let err = provider_at(addr).ensure_token().await.unwrap_err();
        assert!(matches!(err, IamError::Unauthorized));
    }

    // --- find_user ----------------------------------------------------------

    #[tokio::test]
    async fn find_user_returns_some_on_200() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "200 OK",
                r#"{"data":{"id":"u-abc","email":"alice@example.com","username":"alice","firstname":"Alice","lastname":"Smith","enabled":true,"email_verified":true}}"#.into(),
            ),
        ]);

        let user = provider_at(addr)
            .find_user(&IamUserId("u-abc".into()))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(user.id, IamUserId("u-abc".into()));
        assert_eq!(user.email, "alice@example.com");
        // firstname + lastname joined
        assert_eq!(user.name, Some("Alice Smith".into()));
        assert!(user.email_verified);
        assert!(user.enabled);
    }

    #[tokio::test]
    async fn find_user_name_only_firstname() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "200 OK",
                r#"{"data":{"id":"u-1","email":"a@b.com","username":"a","firstname":"Alice","enabled":true,"email_verified":false}}"#.into(),
            ),
        ]);

        let user = provider_at(addr)
            .find_user(&IamUserId("u-1".into()))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(user.name, Some("Alice".into()));
    }

    #[tokio::test]
    async fn find_user_name_none_when_no_names() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "200 OK",
                r#"{"data":{"id":"u-2","email":"b@b.com","username":"b","enabled":false,"email_verified":false}}"#.into(),
            ),
        ]);

        let user = provider_at(addr)
            .find_user(&IamUserId("u-2".into()))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(user.name, None);
    }

    #[tokio::test]
    async fn find_user_returns_none_on_404() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            ("404 Not Found", r#"{"error":"user_not_found"}"#.into()),
        ]);

        let result = provider_at(addr)
            .find_user(&IamUserId("missing".into()))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    // --- find_user_by_email -------------------------------------------------

    #[tokio::test]
    async fn find_user_by_email_returns_some_when_found() {
        // FerrisKey returns all users; we filter client-side by email.
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "200 OK",
                r#"{"data":[{"id":"u-1","email":"bob@example.com","username":"bob","enabled":true,"email_verified":false},{"id":"u-2","email":"carol@example.com","username":"carol","enabled":true,"email_verified":true}]}"#.into(),
            ),
        ]);

        let user = provider_at(addr)
            .find_user_by_email("bob@example.com")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(user.id, IamUserId("u-1".into()));
        assert_eq!(user.username, "bob");
        assert!(!user.email_verified);
    }

    #[tokio::test]
    async fn find_user_by_email_returns_none_when_not_in_list() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "200 OK",
                r#"{"data":[{"id":"u-1","email":"other@example.com","username":"other","enabled":true,"email_verified":false}]}"#.into(),
            ),
        ]);

        let result = provider_at(addr)
            .find_user_by_email("ghost@example.com")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn find_user_by_email_returns_none_for_empty_list() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            ("200 OK", r#"{"data":[]}"#.into()),
        ]);

        let result = provider_at(addr)
            .find_user_by_email("ghost@example.com")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    // --- create_user --------------------------------------------------------

    #[tokio::test]
    async fn create_user_reads_id_from_response_body() {
        // FerrisKey returns { "data": <user> } on create (200 or 201),
        // no Location header.
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "201 Created",
                r#"{"data":{"id":"new-uid","email":"carol@example.com","username":"carol","firstname":"Carol","enabled":true,"email_verified":false}}"#.into(),
            ),
        ]);

        let user = provider_at(addr)
            .create_user(IamCreateUser {
                email: "carol@example.com".into(),
                username: "carol".into(),
                name: Some("Carol".into()),
                send_invite_email: false,
            })
            .await
            .unwrap();

        assert_eq!(user.id, IamUserId("new-uid".into()));
        assert_eq!(user.email, "carol@example.com");
        assert_eq!(user.name, Some("Carol".into()));
    }

    #[tokio::test]
    async fn create_user_accepts_200_status() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "200 OK",
                r#"{"data":{"id":"uid-200","email":"dave@example.com","username":"dave","enabled":true,"email_verified":false}}"#.into(),
            ),
        ]);

        let user = provider_at(addr)
            .create_user(IamCreateUser {
                email: "dave@example.com".into(),
                username: "dave".into(),
                name: None,
                send_invite_email: true, // ignored by FerrisKey adapter
            })
            .await
            .unwrap();

        assert_eq!(user.id, IamUserId("uid-200".into()));
    }

    #[tokio::test]
    async fn create_user_returns_conflict_on_409() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            ("409 Conflict", r#"{"error":"user_already_exists"}"#.into()),
        ]);

        let err = provider_at(addr)
            .create_user(IamCreateUser {
                email: "dup@example.com".into(),
                username: "dup".into(),
                name: None,
                send_invite_email: false,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, IamError::Conflict(_)));
    }

    // --- update_user --------------------------------------------------------

    #[tokio::test]
    async fn update_user_returns_updated_iam_user() {
        // FerrisKey PUT returns { "data": <updated-user> } directly.
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "200 OK",
                r#"{"data":{"id":"u-1","email":"new@example.com","username":"alice","enabled":false,"email_verified":false}}"#.into(),
            ),
        ]);

        let user = provider_at(addr)
            .update_user(
                &IamUserId("u-1".into()),
                IamUpdateUser {
                    email: Some("new@example.com".into()),
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(user.email, "new@example.com");
        assert!(!user.enabled);
    }

    #[tokio::test]
    async fn update_user_disable_returns_disabled_user() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "200 OK",
                r#"{"data":{"id":"u-5","email":"e@e.com","username":"e","enabled":false,"email_verified":false}}"#.into(),
            ),
        ]);

        let user = provider_at(addr)
            .update_user(
                &IamUserId("u-5".into()),
                IamUpdateUser {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert!(!user.enabled);
    }

    #[tokio::test]
    async fn update_user_returns_not_found_on_404() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            ("404 Not Found", r#"{"error":"user_not_found"}"#.into()),
        ]);

        let err = provider_at(addr)
            .update_user(&IamUserId("missing".into()), IamUpdateUser::default())
            .await
            .unwrap_err();

        assert!(matches!(err, IamError::NotFound));
    }

    // --- delete_user --------------------------------------------------------

    #[tokio::test]
    async fn delete_user_returns_ok_on_204() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            ("204 No Content", "".into()),
        ]);

        provider_at(addr)
            .delete_user(&IamUserId("u-del".into()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_user_returns_ok_on_200() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            ("200 OK", "".into()),
        ]);

        provider_at(addr)
            .delete_user(&IamUserId("u-del2".into()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_user_returns_not_found_on_404() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            ("404 Not Found", r#"{"error":"user_not_found"}"#.into()),
        ]);

        let err = provider_at(addr)
            .delete_user(&IamUserId("ghost".into()))
            .await
            .unwrap_err();
        assert!(matches!(err, IamError::NotFound));
    }

    // --- error mapping ------------------------------------------------------

    #[tokio::test]
    async fn find_user_returns_forbidden_on_403() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            ("403 Forbidden", r#"{"error":"forbidden"}"#.into()),
        ]);

        let err = provider_at(addr)
            .find_user(&IamUserId("u-1".into()))
            .await
            .unwrap_err();
        assert!(matches!(err, IamError::Forbidden));
    }

    #[tokio::test]
    async fn find_user_returns_unavailable_on_503() {
        let addr = serve_responses(vec![
            (
                "200 OK",
                r#"{"access_token":"tok","expires_in":300}"#.into(),
            ),
            (
                "503 Service Unavailable",
                r#"{"error":"service_unavailable"}"#.into(),
            ),
        ]);

        let err = provider_at(addr)
            .find_user(&IamUserId("u-1".into()))
            .await
            .unwrap_err();
        assert!(matches!(err, IamError::Unavailable(_)));
    }
}
