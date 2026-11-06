use std::{fmt, future::Future};

use chrono::{DateTime, Duration, Utc};
use common::{CoreError, OrganizationId};
use mestier_macros::transactional;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use super::credential::{open_object, seal_object, secret_names};
use crate::{
    application::MestierUseCase,
    domain::automation::{
        credential::{Credential, CredentialError, CredentialStatus, is_internal_secret},
        credential_kind::credential_kind,
        oauth::{OauthCallback, OauthOutcome, OauthState},
        ports::{CredentialRepository, OauthStateRepository},
    },
    infrastructure::automation::oauth::{
        Grant, OAuth1Settings, OAuth1Token, OAuth2Settings, TokenSet, authorization_url,
        exchange_oauth1_token, fetch_account, oauth1_authorization_url, pkce_challenge,
        random_token, request_oauth1_token, request_tokens, state_hash,
    },
};

pub const MAX_PENDING_AUTHORIZATIONS_PER_CREDENTIAL: i64 = 20;

const EXPIRY_SKEW_SECONDS: i64 = 30;

#[derive(Clone, PartialEq, Eq)]
pub enum TokenOutcome {
    Token(String),
    Failed(String),
}

impl fmt::Debug for TokenOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Token(_) => f.write_str("Token([REDACTED])"),
            Self::Failed(message) => f.debug_tuple("Failed").field(message).finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StartOauthCommand {
    pub org_id: OrganizationId,
    pub credential_id: Uuid,
    pub member_id: Uuid,
    pub redirect_uri: String,
    pub return_to: String,
}

enum Exchange {
    OAuth1 {
        settings: OAuth1Settings,
        access: OAuth1Token,
    },
    OAuth2 {
        settings: OAuth2Settings,
        tokens: TokenSet,
        account: Option<String>,
    },
}

fn conflict(message: impl Into<String>) -> CoreError {
    CoreError::Conflict(message.into())
}

fn changed_meanwhile() -> CoreError {
    conflict("the credential changed while the provider was being called")
}

fn merged(credential: &Credential, secrets: &Map<String, Value>) -> Map<String, Value> {
    let mut fields = credential.data.clone();
    fields.extend(secrets.clone());
    fields
}

fn text_of(object: &Map<String, Value>, name: &str) -> Option<String> {
    object.get(name).and_then(Value::as_str).map(str::to_owned)
}

fn usable_token(
    secrets: &Map<String, Value>,
    expires_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<String> {
    let token = text_of(secrets, "access_token").filter(|token| !token.is_empty())?;
    match expires_at {
        Some(expiry) if expiry <= now + Duration::seconds(EXPIRY_SKEW_SECONDS) => None,
        _ => Some(token),
    }
}

fn require_connectable(credential: &Credential) -> Result<(), CoreError> {
    match credential_kind(&credential.kind) {
        Some(kind) if kind.connectable => Ok(()),
        _ => Err(CredentialError::NotConnectable.into()),
    }
}

impl MestierUseCase {
    async fn provider_call<T>(&self, call: impl Future<Output = T>) -> T {
        let _slot = self.oauth.provider_slot().await;
        call.await
    }

    pub async fn start_oauth_authorization(
        &self,
        command: StartOauthCommand,
    ) -> Result<String, CoreError> {
        let cipher = self.require_cipher()?;
        let (credential, fields) = self
            .prepare_oauth_start(command.org_id, command.credential_id)
            .await?;

        let now = Utc::now();
        let state = random_token(32);
        let (url, payload) = if credential.kind == "oauth1" {
            let settings = OAuth1Settings::from_fields(&fields).map_err(conflict)?;
            let callback = format!("{}?state={state}", command.redirect_uri);
            let request_token = self
                .provider_call(request_oauth1_token(
                    &self.oauth_http,
                    self.oauth_access,
                    &settings,
                    &callback,
                ))
                .await
                .map_err(|error| provider_unavailable(&credential, &error))?;
            let url =
                oauth1_authorization_url(&settings, &request_token.token).map_err(conflict)?;
            let payload = seal_object(
                &cipher,
                json!({
                    "oauth_token": request_token.token,
                    "oauth_token_secret": request_token.secret,
                })
                .as_object()
                .expect("a literal object"),
            )?;
            (url, Some(payload))
        } else {
            let settings = OAuth2Settings::from_fields(&fields).map_err(conflict)?;
            let verifier = settings.pkce.then(|| random_token(32));
            let challenge = verifier.as_deref().map(pkce_challenge);
            let url = authorization_url(
                &settings,
                &command.redirect_uri,
                &state,
                challenge.as_deref(),
            )
            .map_err(conflict)?;
            let payload = match &verifier {
                Some(verifier) => Some(seal_object(
                    &cipher,
                    json!({ "code_verifier": verifier })
                        .as_object()
                        .expect("a literal object"),
                )?),
                None => None,
            };
            (url, payload)
        };

        self.store_oauth_state(
            state_hash(&state),
            OauthState::new(
                command.org_id,
                credential.id,
                command.member_id,
                command.redirect_uri,
                command.return_to,
                payload,
                now,
            ),
        )
        .await?;
        Ok(url.to_string())
    }

    #[transactional(credential, oauth_state)]
    async fn prepare_oauth_start(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<(Credential, Map<String, Value>), CoreError> {
        let cipher = self.require_cipher()?;
        let mut credentials = credential_repository;
        let mut states = oauth_state_repository;

        let (credential, sealed) = credentials
            .find_sealed_by_id(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        require_connectable(&credential)?;
        states.purge_expired(Utc::now()).await?;
        if states.count_for_credential(credential.id).await?
            >= MAX_PENDING_AUTHORIZATIONS_PER_CREDENTIAL
        {
            return Err(CredentialError::TooManyAuthorizations.into());
        }
        let fields = merged(&credential, &open_object(&cipher, &sealed)?);
        Ok((credential, fields))
    }

    #[transactional(oauth_state)]
    async fn store_oauth_state(&self, hash: Vec<u8>, state: OauthState) -> Result<(), CoreError> {
        let mut states = oauth_state_repository;
        states.purge_expired(state.created_at).await?;
        if states.count_for_credential(state.credential_id).await?
            >= MAX_PENDING_AUTHORIZATIONS_PER_CREDENTIAL
        {
            return Err(CredentialError::TooManyAuthorizations.into());
        }
        states.insert(&hash, &state).await
    }

    #[transactional(oauth_state)]
    async fn consume_oauth_state(
        &self,
        hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<OauthState>, CoreError> {
        let mut states = oauth_state_repository;
        states.consume(hash, now).await
    }

    pub async fn complete_oauth_authorization(
        &self,
        callback: OauthCallback,
    ) -> Result<OauthOutcome, CoreError> {
        let Some(state) = self
            .consume_oauth_state(&state_hash(&callback.state), Utc::now())
            .await?
        else {
            return Err(CoreError::NotFound);
        };

        let connected = if callback.provider_refused {
            false
        } else {
            match self.connect_with_callback(&state, &callback).await {
                Ok(()) => true,
                Err(error) => {
                    tracing::warn!(
                        org_id = %state.org_id.0,
                        credential_id = %state.credential_id,
                        %error,
                        "an OAuth authorization could not be completed"
                    );
                    false
                }
            }
        };

        Ok(OauthOutcome {
            return_to: state.return_to,
            connected,
        })
    }

    async fn connect_with_callback(
        &self,
        state: &OauthState,
        callback: &OauthCallback,
    ) -> Result<(), CoreError> {
        let cipher = self.require_cipher()?;
        let _flight = self.oauth.flight(state.credential_id).lock_owned().await;
        let (credential, secrets) = self
            .read_oauth_credential(state.org_id, state.credential_id)
            .await?;
        require_connectable(&credential)?;
        let fields = merged(&credential, &secrets);
        let payload = match &state.payload {
            Some(payload) => open_object(&cipher, payload)?,
            None => Map::new(),
        };

        let exchange = if credential.kind == "oauth1" {
            let settings = OAuth1Settings::from_fields(&fields).map_err(conflict)?;
            let verifier = callback
                .oauth_verifier
                .as_deref()
                .ok_or_else(|| conflict("the provider sent no verifier"))?;
            let stored = OAuth1Token {
                token: text_of(&payload, "oauth_token").unwrap_or_default(),
                secret: text_of(&payload, "oauth_token_secret").unwrap_or_default(),
            };
            if stored.token.is_empty() || callback.oauth_token.as_deref() != Some(&stored.token) {
                return Err(conflict("the provider answered for another request token"));
            }
            let access = self
                .provider_call(exchange_oauth1_token(
                    &self.oauth_http,
                    self.oauth_access,
                    &settings,
                    &stored,
                    verifier,
                ))
                .await
                .map_err(|error| provider_unavailable(&credential, &error))?;
            Exchange::OAuth1 { settings, access }
        } else {
            let settings = OAuth2Settings::from_fields(&fields).map_err(conflict)?;
            let code = callback
                .code
                .as_deref()
                .ok_or_else(|| conflict("the provider sent no code"))?;
            let verifier = text_of(&payload, "code_verifier");
            let userinfo_url = settings.userinfo_url.clone();
            let (tokens, account) = self
                .provider_call(async {
                    let tokens = request_tokens(
                        &self.oauth_http,
                        self.oauth_access,
                        &settings,
                        Grant::AuthorizationCode {
                            code,
                            redirect_uri: &state.redirect_uri,
                            verifier: verifier.as_deref(),
                        },
                    )
                    .await?;
                    let account = match &userinfo_url {
                        Some(url) => {
                            fetch_account(
                                &self.oauth_http,
                                self.oauth_access,
                                url,
                                &tokens.access_token,
                            )
                            .await
                        }
                        None => None,
                    };
                    Ok::<_, String>((tokens, account))
                })
                .await
                .map_err(|error| provider_unavailable(&credential, &error))?;
            Exchange::OAuth2 {
                settings,
                tokens,
                account,
            }
        };

        self.write_connection(state.org_id, state.credential_id, exchange, Utc::now())
            .await
    }

    #[transactional(credential)]
    async fn read_oauth_credential(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<(Credential, Map<String, Value>), CoreError> {
        let cipher = self.require_cipher()?;
        let mut credentials = credential_repository;
        let (credential, sealed) = credentials
            .find_sealed_by_id(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let secrets = open_object(&cipher, &sealed)?;
        Ok((credential, secrets))
    }

    #[transactional(credential)]
    async fn write_connection(
        &self,
        org_id: OrganizationId,
        id: Uuid,
        exchange: Exchange,
        now: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let cipher = self.require_cipher()?;
        let mut credentials = credential_repository;
        let (credential, sealed) = credentials
            .find_sealed_for_update(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        require_connectable(&credential)?;
        let mut secrets = open_object(&cipher, &sealed)?;
        let fields = merged(&credential, &secrets);
        let mut updated = credential.clone();

        match exchange {
            Exchange::OAuth1 { settings, access } => {
                if OAuth1Settings::from_fields(&fields).map_err(conflict)? != settings {
                    return Err(changed_meanwhile());
                }
                secrets.insert("oauth_token".to_owned(), Value::String(access.token));
                secrets.insert(
                    "oauth_token_secret".to_owned(),
                    Value::String(access.secret),
                );
                updated.expires_at = None;
            }
            Exchange::OAuth2 {
                settings,
                tokens,
                account,
            } => {
                if OAuth2Settings::from_fields(&fields).map_err(conflict)? != settings {
                    return Err(changed_meanwhile());
                }
                updated.account = account;
                updated.expires_at = tokens.expires_at(now);
                match tokens.refresh_token {
                    Some(refresh) => {
                        secrets.insert("refresh_token".to_owned(), Value::String(refresh))
                    }
                    None => secrets.remove("refresh_token"),
                };
                secrets.insert(
                    "access_token".to_owned(),
                    Value::String(tokens.access_token),
                );
            }
        }

        updated.status = CredentialStatus::Valid;
        updated.secrets_set = secret_names(&secrets);
        let resealed = seal_object(&cipher, &secrets)?;
        credentials.store_tokens(org_id, &updated, &resealed).await
    }

    #[transactional(credential)]
    pub async fn disconnect_credential_account(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Credential, CoreError> {
        let cipher = self.require_cipher()?;
        let mut credentials = credential_repository;
        let (credential, sealed) = credentials
            .find_sealed_for_update(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        require_connectable(&credential)?;

        let mut secrets = open_object(&cipher, &sealed)?;
        secrets.retain(|name, _| !is_internal_secret(name));
        let updated = Credential {
            status: CredentialStatus::NotConnected,
            account: None,
            expires_at: None,
            secrets_set: secret_names(&secrets),
            ..credential
        };
        let resealed = seal_object(&cipher, &secrets)?;
        credentials.update(org_id, &updated, Some(&resealed)).await
    }

    pub async fn oauth_access_token(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<String, String> {
        let _flight = self.oauth.flight(id).lock_owned().await;
        match self.renewed_oauth_token(org_id, id).await {
            Ok(TokenOutcome::Token(token)) => Ok(token),
            Ok(TokenOutcome::Failed(message)) => Err(message),
            Err(CoreError::NotFound) => Err(self.missing_credential_message(org_id, id).await),
            Err(error) => Err(format!("the OAuth credential is unusable: {error}")),
        }
    }

    async fn renewed_oauth_token(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<TokenOutcome, CoreError> {
        let (credential, secrets) = self.read_oauth_credential(org_id, id).await?;
        let now = Utc::now();
        let name = credential.name.clone();

        let (settings, refresh) = match credential.kind.as_str() {
            "oauth2_client_credentials" => {
                if let Some(token) = usable_token(&secrets, credential.expires_at, now) {
                    return Ok(TokenOutcome::Token(token));
                }
                match OAuth2Settings::from_fields(&merged(&credential, &secrets)) {
                    Ok(settings) => (settings, None),
                    Err(error) => return Ok(TokenOutcome::Failed(error)),
                }
            }
            "oauth2_authorization_code" => {
                if credential.status == CredentialStatus::NotConnected
                    || text_of(&secrets, "access_token").is_none()
                {
                    return Ok(TokenOutcome::Failed(format!(
                        "credential `{name}` is not connected: connect an account first"
                    )));
                }
                if let Some(token) = usable_token(&secrets, credential.expires_at, now)
                    && credential.status != CredentialStatus::Expired
                {
                    return Ok(TokenOutcome::Token(token));
                }
                let Some(refresh) = text_of(&secrets, "refresh_token") else {
                    return Ok(TokenOutcome::Failed(format!(
                        "credential `{name}` has expired and has no refresh token: reconnect the account"
                    )));
                };
                match OAuth2Settings::from_fields(&merged(&credential, &secrets)) {
                    Ok(settings) => (settings, Some(refresh)),
                    Err(error) => return Ok(TokenOutcome::Failed(error)),
                }
            }
            other => {
                return Ok(TokenOutcome::Failed(format!(
                    "credential `{name}` of kind `{other}` has no token to fetch"
                )));
            }
        };

        let grant = match &refresh {
            Some(refresh_token) => Grant::Refresh { refresh_token },
            None => Grant::ClientCredentials,
        };
        let outcome = self
            .provider_call(request_tokens(
                &self.oauth_http,
                self.oauth_access,
                &settings,
                grant,
            ))
            .await;
        self.store_renewed_token(org_id, id, settings, refresh, outcome, now)
            .await
    }

    #[transactional(credential)]
    async fn store_renewed_token(
        &self,
        org_id: OrganizationId,
        id: Uuid,
        used: OAuth2Settings,
        used_refresh: Option<String>,
        outcome: Result<TokenSet, String>,
        now: DateTime<Utc>,
    ) -> Result<TokenOutcome, CoreError> {
        let cipher = self.require_cipher()?;
        let mut credentials = credential_repository;
        let (credential, sealed) = credentials
            .find_sealed_for_update(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let mut secrets = open_object(&cipher, &sealed)?;
        let name = credential.name.clone();

        let unchanged = OAuth2Settings::from_fields(&merged(&credential, &secrets))
            .is_ok_and(|current| current == used)
            && text_of(&secrets, "refresh_token") == used_refresh;
        if !unchanged {
            return Ok(TokenOutcome::Failed(format!(
                "credential `{name}` changed while its token was being renewed: try again"
            )));
        }

        match outcome {
            Ok(fresh) => {
                secrets.insert(
                    "access_token".to_owned(),
                    Value::String(fresh.access_token.clone()),
                );
                if let Some(refresh) = &fresh.refresh_token {
                    secrets.insert("refresh_token".to_owned(), Value::String(refresh.clone()));
                }
                let stored = Credential {
                    status: CredentialStatus::Valid,
                    expires_at: fresh.expires_at(now),
                    secrets_set: secret_names(&secrets),
                    ..credential
                };
                let resealed = seal_object(&cipher, &secrets)?;
                credentials.store_tokens(org_id, &stored, &resealed).await?;
                Ok(TokenOutcome::Token(fresh.access_token))
            }
            Err(error) if used_refresh.is_some() => {
                let expired = Credential {
                    status: CredentialStatus::Expired,
                    ..credential
                };
                credentials.store_tokens(org_id, &expired, &sealed).await?;
                Ok(TokenOutcome::Failed(format!(
                    "refreshing the token of credential `{name}` failed: {error}"
                )))
            }
            Err(error) => Ok(TokenOutcome::Failed(format!(
                "fetching a token for credential `{name}` failed: {error}"
            ))),
        }
    }
}

fn provider_unavailable(credential: &Credential, error: &str) -> CoreError {
    tracing::warn!(
        org_id = %credential.org_id.0,
        credential_id = %credential.id,
        error,
        "an OAuth provider call failed"
    );
    CredentialError::ProviderUnavailable.into()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::Duration as StdDuration,
    };

    use common::generate_uuid_v7;
    use reqwest::{Client, Url};
    use sqlx::{PgPool, postgres::PgPoolOptions};

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::credential::{
        CreateCredentialCommand, CredentialOrigin, UpdateCredentialCommand,
    };
    use crate::domain::automation::secret::SealedSecret;
    use crate::infrastructure::automation::connectors::{
        http_client::{HttpAuth, open_http_auth},
        test_support::{CapturedRequest, StubResponse, TestServer},
    };
    use crate::infrastructure::automation::webhook::secret::SecretCipher;
    use crate::infrastructure::realtime::EventHub;

    async fn seed_organization(pool: &PgPool, label: &str) -> (OrganizationId, Uuid) {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        (OrganizationId(org_id), owner_id)
    }

    fn plain_client() -> Client {
        Client::builder().no_proxy().build().unwrap()
    }

    fn use_case(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
            .with_cipher(Some(Arc::new(SecretCipher::new(&[7u8; 32]).unwrap())))
            .with_oauth_http_client(plain_client())
    }

    fn form_of(request: &CapturedRequest) -> HashMap<String, String> {
        let body = String::from_utf8_lossy(&request.body).into_owned();
        Url::parse(&format!("http://provider.invalid/?{body}"))
            .unwrap()
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect()
    }

    #[derive(Default)]
    struct Behaviour {
        refresh_fails: AtomicBool,
        refresh_delay_ms: AtomicUsize,
        code_delay_ms: AtomicUsize,
        refresh_calls: AtomicUsize,
        client_credential_calls: AtomicUsize,
        code_calls: AtomicUsize,
        omit_refresh_token: AtomicBool,
        expires_in: Mutex<Option<Value>>,
        in_flight: AtomicUsize,
        most_in_flight: AtomicUsize,
        pool: Mutex<Option<PgPool>>,
        connections_in_use: Mutex<Vec<usize>>,
        forms: Mutex<Vec<HashMap<String, String>>>,
        headers: Mutex<Vec<HashMap<String, String>>>,
    }

    impl Behaviour {
        fn expiry(&self, default: Value) -> Value {
            self.expires_in.lock().unwrap().clone().unwrap_or(default)
        }

        fn linger(&self, delay_ms: usize) {
            let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            self.most_in_flight.fetch_max(now, Ordering::SeqCst);
            std::thread::sleep(StdDuration::from_millis(delay_ms as u64));
            if let Some(pool) = self.pool.lock().unwrap().as_ref() {
                self.connections_in_use
                    .lock()
                    .unwrap()
                    .push(pool.size() as usize - pool.num_idle());
            }
            self.in_flight.fetch_sub(1, Ordering::SeqCst);
        }
    }

    async fn provider(behaviour: Arc<Behaviour>) -> TestServer {
        TestServer::start(move |request| {
            let behaviour = &behaviour;
            match request.path.as_str() {
                "/token" => {
                    let form = form_of(request);
                    behaviour.forms.lock().unwrap().push(form.clone());
                    behaviour.headers.lock().unwrap().push(request.headers.clone());
                    match form.get("grant_type").map(String::as_str) {
                        Some("authorization_code") => {
                            behaviour.code_calls.fetch_add(1, Ordering::SeqCst);
                            behaviour.linger(behaviour.code_delay_ms.load(Ordering::SeqCst));
                            if form.get("code").map(String::as_str) != Some("good-code") {
                                return StubResponse::json(400, &json!({ "error": "invalid_grant" }));
                            }
                            let mut body = json!({ "access_token": "AT-code", "expires_in": behaviour.expiry(json!(3600)) });
                            if !behaviour.omit_refresh_token.load(Ordering::SeqCst) {
                                body["refresh_token"] = json!("RT-code");
                            }
                            StubResponse::json(200, &body)
                        }
                        Some("refresh_token") => {
                            let call = behaviour.refresh_calls.fetch_add(1, Ordering::SeqCst) + 1;
                            behaviour.linger(behaviour.refresh_delay_ms.load(Ordering::SeqCst));
                            if behaviour.refresh_fails.load(Ordering::SeqCst) {
                                return StubResponse::json(400, &json!({ "error": "invalid_grant" }));
                            }
                            StubResponse::json(
                                200,
                                &json!({ "access_token": format!("AT-refresh-{call}"), "expires_in": behaviour.expiry(json!("3600")) }),
                            )
                        }
                        Some("client_credentials") => {
                            let call = behaviour.client_credential_calls.fetch_add(1, Ordering::SeqCst) + 1;
                            behaviour.linger(behaviour.refresh_delay_ms.load(Ordering::SeqCst));
                            StubResponse::json(
                                200,
                                &json!({ "access_token": format!("CC-{call}"), "expires_in": behaviour.expiry(json!(3600)) }),
                            )
                        }
                        _ => StubResponse::json(400, &json!({ "error": "unsupported_grant_type" })),
                    }
                }
                "/userinfo" => {
                    if request.headers.get("authorization").map(String::as_str) == Some("Bearer AT-code") {
                        StubResponse::json(200, &json!({ "email": "ada@example.com", "sub": "42" }))
                    } else {
                        StubResponse::json(401, &json!({ "error": "invalid_token" }))
                    }
                }
                _ => StubResponse::status(404),
            }
        })
        .await
    }

    async fn code_credential(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        server: &TestServer,
        name: &str,
    ) -> Credential {
        usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "oauth2_authorization_code".to_owned(),
                name: name.to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(json!({
                    "authorization_url": format!("{}/authorize", server.url),
                    "token_url": format!("{}/token", server.url),
                    "userinfo_url": format!("{}/userinfo", server.url),
                    "client_id": "mestier-client",
                    "client_secret": "client-secret-value",
                    "scopes": "openid profile",
                })),
            })
            .await
            .unwrap()
            .0
    }

    async fn client_credentials_credential(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        server: &TestServer,
        authentication: &str,
    ) -> Credential {
        usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "oauth2_client_credentials".to_owned(),
                name: format!("Service {authentication}"),
                origin: CredentialOrigin::Supplied,
                data: Some(json!({
                    "token_url": format!("{}/token", server.url),
                    "client_id": "svc",
                    "client_secret": "svc-secret",
                    "scopes": "read",
                    "client_authentication": authentication,
                })),
            })
            .await
            .unwrap()
            .0
    }

    fn start_command(
        org_id: OrganizationId,
        credential_id: Uuid,
        member_id: Uuid,
    ) -> StartOauthCommand {
        StartOauthCommand {
            org_id,
            credential_id,
            member_id,
            redirect_uri: "https://api.example.com/oauth/callback".to_owned(),
            return_to: "/o/atelier/automatisation/credentials".to_owned(),
        }
    }

    fn state_of(url: &str) -> String {
        Url::parse(url)
            .unwrap()
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned())
            .expect("the authorization url carries a state")
    }

    fn param_of(url: &str, name: &str) -> Option<String> {
        Url::parse(url)
            .unwrap()
            .query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    }

    fn callback(state: &str, code: &str) -> OauthCallback {
        OauthCallback {
            state: state.to_owned(),
            code: Some(code.to_owned()),
            ..OauthCallback::default()
        }
    }

    async fn sealed_secrets(
        pool: &PgPool,
        usecase: &MestierUseCase,
        id: Uuid,
    ) -> Map<String, Value> {
        let row = sqlx::query!(
            "SELECT data_nonce, data_ciphertext FROM automation.credential WHERE id = $1",
            id
        )
        .fetch_one(pool)
        .await
        .unwrap();
        let cipher = usecase.cipher.clone().unwrap();
        open_object(
            &cipher,
            &SealedSecret {
                nonce: row.data_nonce,
                ciphertext: row.data_ciphertext,
            },
        )
        .unwrap()
    }

    async fn connect(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        member: Uuid,
        credential: &Credential,
    ) {
        let url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();
        let outcome = usecase
            .complete_oauth_authorization(callback(&state_of(&url), "good-code"))
            .await
            .unwrap();
        assert!(outcome.connected);
    }

    async fn expire_access_token(pool: &PgPool, id: Uuid) {
        sqlx::query!(
            "UPDATE automation.credential SET expires_at = now() - interval '1 hour' WHERE id = $1",
            id
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_authorization_code_flow_stores_sealed_tokens_shows_the_account_and_disconnects() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-flow").await;
        let behaviour = Arc::new(Behaviour::default());
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte Google").await;
        assert_eq!(credential.status, CredentialStatus::NotConnected);

        let url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();

        assert!(
            url.starts_with(&format!("{}/authorize?", server.url)),
            "{url}"
        );
        assert_eq!(param_of(&url, "response_type").as_deref(), Some("code"));
        assert_eq!(
            param_of(&url, "client_id").as_deref(),
            Some("mestier-client")
        );
        assert_eq!(
            param_of(&url, "redirect_uri").as_deref(),
            Some("https://api.example.com/oauth/callback")
        );
        assert_eq!(param_of(&url, "scope").as_deref(), Some("openid profile"));
        assert_eq!(
            param_of(&url, "code_challenge_method").as_deref(),
            Some("S256")
        );
        assert!(!url.contains("client-secret-value"));
        let state = state_of(&url);
        assert!(
            state.len() >= 43,
            "256 bits of base64url, got {}",
            state.len()
        );

        let row = sqlx::query!(
            "SELECT state_hash, org_id, credential_id, member_id, expires_at - created_at AS \"window!\"
             FROM automation.oauth_state WHERE credential_id = $1",
            credential.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.org_id, org_id.0);
        assert_eq!(row.member_id, member);
        assert_ne!(
            row.state_hash,
            state.as_bytes(),
            "the raw state is never stored"
        );
        assert_eq!(row.state_hash, state_hash(&state));

        let outcome = usecase
            .complete_oauth_authorization(callback(&state, "good-code"))
            .await
            .unwrap();

        assert!(outcome.connected);
        assert_eq!(outcome.return_to, "/o/atelier/automatisation/credentials");
        let connected = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(connected.status, CredentialStatus::Valid);
        assert_eq!(connected.status_at(Utc::now()), CredentialStatus::Valid);
        assert_eq!(connected.account.as_deref(), Some("ada@example.com"));
        assert!(connected.expires_at.unwrap() > Utc::now() + Duration::minutes(50));
        assert_eq!(
            connected.secrets_set,
            vec!["access_token", "client_secret", "refresh_token"]
        );
        assert!(!connected.data.contains_key("access_token"));
        let sealed = sealed_secrets(&pool, &usecase, credential.id).await;
        assert_eq!(sealed["access_token"], json!("AT-code"));
        assert_eq!(sealed["refresh_token"], json!("RT-code"));

        let exchanged = behaviour.forms.lock().unwrap()[0].clone();
        assert_eq!(exchanged["grant_type"], "authorization_code");
        assert_eq!(
            exchanged["redirect_uri"],
            "https://api.example.com/oauth/callback"
        );
        assert_eq!(
            pkce_challenge(&exchanged["code_verifier"]),
            param_of(&url, "code_challenge").unwrap(),
            "the verifier answers the challenge sent at start"
        );
        assert_eq!(exchanged["client_id"], "mestier-client");
        assert!(
            !exchanged.values().any(|value| *value == state),
            "the state never goes to the provider"
        );

        assert_eq!(
            usecase
                .oauth_access_token(org_id, credential.id)
                .await
                .unwrap(),
            "AT-code"
        );
        assert_eq!(behaviour.refresh_calls.load(Ordering::SeqCst), 0);
        match open_http_auth(&usecase, org_id, credential.id)
            .await
            .unwrap()
        {
            HttpAuth::Bearer { token } => assert_eq!(token, "AT-code"),
            other => panic!("an OAuth2 credential applies as a bearer: {other:?}"),
        }

        let disconnected = usecase
            .disconnect_credential_account(org_id, credential.id)
            .await
            .unwrap();
        assert_eq!(disconnected.status, CredentialStatus::NotConnected);
        assert_eq!(disconnected.account, None);
        assert_eq!(disconnected.expires_at, None);
        assert_eq!(disconnected.secrets_set, vec!["client_secret".to_owned()]);
        let sealed = sealed_secrets(&pool, &usecase, credential.id).await;
        assert!(!sealed.contains_key("access_token") && !sealed.contains_key("refresh_token"));
        let error = usecase
            .oauth_access_token(org_id, credential.id)
            .await
            .unwrap_err();
        assert!(
            error.contains("Compte Google") && error.contains("not connected"),
            "{error}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_state_answers_once_and_a_replay_changes_nothing() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-replay").await;
        let behaviour = Arc::new(Behaviour::default());
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;
        let url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();
        let state = state_of(&url);
        assert!(
            usecase
                .complete_oauth_authorization(callback(&state, "good-code"))
                .await
                .unwrap()
                .connected
        );
        usecase
            .disconnect_credential_account(org_id, credential.id)
            .await
            .unwrap();

        let replay = usecase
            .complete_oauth_authorization(callback(&state, "good-code"))
            .await;

        assert!(matches!(replay, Err(CoreError::NotFound)), "{replay:?}");
        assert_eq!(behaviour.code_calls.load(Ordering::SeqCst), 1);
        let after = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            after.status,
            CredentialStatus::NotConnected,
            "a replay reconnects nothing"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_expired_unknown_or_altered_state_is_refused_without_a_call_to_the_provider() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-refused").await;
        let behaviour = Arc::new(Behaviour::default());
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;
        let url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();
        let state = state_of(&url);
        sqlx::query!(
            "UPDATE automation.oauth_state
             SET created_at = now() - interval '11 minutes', expires_at = now() - interval '1 minute'
             WHERE credential_id = $1",
            credential.id
        )
        .execute(&pool)
        .await
        .unwrap();
        let mut altered = state.clone();
        altered.replace_range(0..1, if state.starts_with('A') { "B" } else { "A" });

        for refused in [state.as_str(), altered.as_str(), "unknown", ""] {
            let outcome = usecase
                .complete_oauth_authorization(callback(refused, "good-code"))
                .await;

            assert!(
                matches!(outcome, Err(CoreError::NotFound)),
                "{refused}: {outcome:?}"
            );
        }

        assert_eq!(behaviour.code_calls.load(Ordering::SeqCst), 0);
        let after = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(after.status, CredentialStatus::NotConnected);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_database_refuses_a_state_that_lives_longer_than_ten_minutes_or_a_raw_hash() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-constraint").await;
        let server = provider(Arc::new(Behaviour::default())).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;

        let too_long = sqlx::query!(
            r#"INSERT INTO automation.oauth_state
                   (state_hash, org_id, credential_id, member_id, redirect_uri, return_to, created_at, expires_at)
               VALUES ($1, $2, $3, $4, 'https://a/oauth/callback', '/', now(), now() + interval '11 minutes')"#,
            state_hash("long"),
            org_id.0,
            credential.id,
            member,
        )
        .execute(&pool)
        .await;
        let short_hash = sqlx::query!(
            r#"INSERT INTO automation.oauth_state
                   (state_hash, org_id, credential_id, member_id, redirect_uri, return_to, created_at, expires_at)
               VALUES ($1, $2, $3, $4, 'https://a/oauth/callback', '/', now(), now() + interval '5 minutes')"#,
            b"raw-state".to_vec(),
            org_id.0,
            credential.id,
            member,
        )
        .execute(&pool)
        .await;
        let duplicate = {
            let hash = state_hash("twice");
            for _ in 0..1 {
                sqlx::query!(
                    r#"INSERT INTO automation.oauth_state
                           (state_hash, org_id, credential_id, member_id, redirect_uri, return_to, created_at, expires_at)
                       VALUES ($1, $2, $3, $4, 'https://a/oauth/callback', '/', now(), now() + interval '5 minutes')"#,
                    hash,
                    org_id.0,
                    credential.id,
                    member,
                )
                .execute(&pool)
                .await
                .unwrap();
            }
            sqlx::query!(
                r#"INSERT INTO automation.oauth_state
                       (state_hash, org_id, credential_id, member_id, redirect_uri, return_to, created_at, expires_at)
                   VALUES ($1, $2, $3, $4, 'https://a/oauth/callback', '/', now(), now() + interval '5 minutes')"#,
                hash,
                org_id.0,
                credential.id,
                member,
            )
            .execute(&pool)
            .await
        };

        assert!(too_long.is_err());
        assert!(short_hash.is_err());
        assert!(duplicate.is_err());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_state_connects_only_the_credential_and_organization_it_was_issued_for() {
        let pool = automation_pool().await;
        let (org_a, member_a) = seed_organization(&pool, "oauth-bound-a").await;
        let (org_b, _) = seed_organization(&pool, "oauth-bound-b").await;
        let behaviour = Arc::new(Behaviour::default());
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let mine = code_credential(&usecase, org_a, &server, "Mine").await;
        let sibling = code_credential(&usecase, org_a, &server, "Sibling").await;
        let foreign = code_credential(&usecase, org_b, &server, "Foreign").await;

        let own_url = usecase
            .start_oauth_authorization(start_command(org_a, mine.id, member_a))
            .await
            .unwrap();
        let wrong_org = usecase
            .start_oauth_authorization(start_command(org_a, foreign.id, member_a))
            .await;
        assert!(
            matches!(wrong_org, Err(CoreError::NotFound)),
            "{wrong_org:?}"
        );

        let outcome = usecase
            .complete_oauth_authorization(callback(&state_of(&own_url), "good-code"))
            .await
            .unwrap();

        assert!(outcome.connected);
        let status = |org, id| {
            let usecase = usecase.clone();
            async move {
                usecase
                    .find_credential(org, id)
                    .await
                    .unwrap()
                    .unwrap()
                    .status
            }
        };
        assert_eq!(status(org_a, mine.id).await, CredentialStatus::Valid);
        assert_eq!(
            status(org_a, sibling.id).await,
            CredentialStatus::NotConnected
        );
        assert_eq!(
            status(org_b, foreign.id).await,
            CredentialStatus::NotConnected
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_the_credential_voids_its_pending_authorizations() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-deleted").await;
        let behaviour = Arc::new(Behaviour::default());
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;
        let url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();
        usecase
            .delete_credential(org_id, credential.id)
            .await
            .unwrap();

        let outcome = usecase
            .complete_oauth_authorization(callback(&state_of(&url), "good-code"))
            .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)), "{outcome:?}");
        assert_eq!(behaviour.code_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_refused_authorization_or_a_rejected_code_redirects_as_an_error_and_uses_the_state_up()
     {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-error").await;
        let behaviour = Arc::new(Behaviour::default());
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;

        let denied_url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();
        let denied = usecase
            .complete_oauth_authorization(OauthCallback {
                state: state_of(&denied_url),
                provider_refused: true,
                ..OauthCallback::default()
            })
            .await
            .unwrap();
        assert!(!denied.connected);
        assert_eq!(behaviour.code_calls.load(Ordering::SeqCst), 0);

        let rejected_url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();
        let rejected = usecase
            .complete_oauth_authorization(callback(&state_of(&rejected_url), "bad-code"))
            .await
            .unwrap();
        assert!(!rejected.connected);
        assert_eq!(rejected.return_to, "/o/atelier/automatisation/credentials");
        let retry = usecase
            .complete_oauth_authorization(callback(&state_of(&rejected_url), "good-code"))
            .await;
        assert!(matches!(retry, Err(CoreError::NotFound)));
        let after = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(after.status, CredentialStatus::NotConnected);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_expired_access_token_is_refreshed_once_under_concurrency() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-refresh").await;
        let behaviour = Arc::new(Behaviour::default());
        behaviour.refresh_delay_ms.store(200, Ordering::SeqCst);
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;
        connect(&usecase, org_id, member, &credential).await;
        expire_access_token(&pool, credential.id).await;
        let listed = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            listed.status_at(Utc::now()),
            CredentialStatus::Valid,
            "a lapsed token that can be refreshed is not listed as expired"
        );

        let mut calls = tokio::task::JoinSet::new();
        for _ in 0..8 {
            let usecase = usecase.clone();
            calls.spawn(async move { usecase.oauth_access_token(org_id, credential.id).await });
        }
        let mut tokens = Vec::new();
        while let Some(joined) = calls.join_next().await {
            tokens.push(joined.unwrap().unwrap());
        }

        assert_eq!(
            behaviour.refresh_calls.load(Ordering::SeqCst),
            1,
            "one refresh for eight callers"
        );
        assert!(
            tokens.iter().all(|token| token == "AT-refresh-1"),
            "{tokens:?}"
        );
        let sealed = sealed_secrets(&pool, &usecase, credential.id).await;
        assert_eq!(sealed["access_token"], json!("AT-refresh-1"));
        assert_eq!(
            sealed["refresh_token"],
            json!("RT-code"),
            "the refresh token is kept"
        );
        let refreshed = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert!(refreshed.expires_at.unwrap() > Utc::now() + Duration::minutes(50));
        let form = behaviour.forms.lock().unwrap().last().unwrap().clone();
        assert_eq!(form["refresh_token"], "RT-code");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_failed_refresh_expires_the_credential_naming_it_and_a_later_success_revives_it() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-refresh-fail").await;
        let behaviour = Arc::new(Behaviour::default());
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte Google").await;
        connect(&usecase, org_id, member, &credential).await;
        expire_access_token(&pool, credential.id).await;
        behaviour.refresh_fails.store(true, Ordering::SeqCst);

        let error = usecase
            .oauth_access_token(org_id, credential.id)
            .await
            .unwrap_err();

        assert!(
            error.contains("Compte Google") && error.contains("refreshing"),
            "{error}"
        );
        assert!(error.contains("invalid_grant"), "{error}");
        let after = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            after.status,
            CredentialStatus::Expired,
            "the failure is stored, not rolled back"
        );
        assert_eq!(after.status_at(Utc::now()), CredentialStatus::Expired);

        behaviour.refresh_fails.store(false, Ordering::SeqCst);
        let token = usecase
            .oauth_access_token(org_id, credential.id)
            .await
            .unwrap();
        assert!(token.starts_with("AT-refresh-"), "{token}");
        let revived = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(revived.status, CredentialStatus::Valid);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_lapsed_token_without_a_refresh_token_is_expired_and_never_refreshed() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-no-refresh").await;
        let behaviour = Arc::new(Behaviour::default());
        behaviour.omit_refresh_token.store(true, Ordering::SeqCst);
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;
        connect(&usecase, org_id, member, &credential).await;
        expire_access_token(&pool, credential.id).await;

        let listed = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        let error = usecase
            .oauth_access_token(org_id, credential.id)
            .await
            .unwrap_err();

        assert_eq!(listed.status_at(Utc::now()), CredentialStatus::Expired);
        assert!(error.contains("no refresh token"), "{error}");
        assert_eq!(behaviour.refresh_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn client_credentials_cache_their_token_until_it_lapses_then_renew_it_once_under_concurrency()
     {
        let pool = automation_pool().await;
        let (org_id, _) = seed_organization(&pool, "oauth-client-credentials").await;
        let behaviour = Arc::new(Behaviour::default());
        behaviour.refresh_delay_ms.store(200, Ordering::SeqCst);
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let by_header = client_credentials_credential(&usecase, org_id, &server, "header").await;
        let by_body = client_credentials_credential(&usecase, org_id, &server, "body").await;

        let first = usecase
            .oauth_access_token(org_id, by_header.id)
            .await
            .unwrap();
        let second = usecase
            .oauth_access_token(org_id, by_header.id)
            .await
            .unwrap();

        assert_eq!(first, "CC-1");
        assert_eq!(second, "CC-1", "a cached token is not fetched again");
        assert_eq!(behaviour.client_credential_calls.load(Ordering::SeqCst), 1);
        let sealed = sealed_secrets(&pool, &usecase, by_header.id).await;
        assert_eq!(sealed["access_token"], json!("CC-1"));
        let stored = usecase
            .find_credential(org_id, by_header.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.status_at(Utc::now()), CredentialStatus::Valid);
        let basic = behaviour.headers.lock().unwrap()[0]
            .get("authorization")
            .cloned()
            .unwrap();
        assert!(basic.starts_with("Basic "), "{basic}");
        let first_form = behaviour.forms.lock().unwrap()[0].clone();
        assert_eq!(first_form["grant_type"], "client_credentials");
        assert_eq!(first_form["scope"], "read");
        assert!(
            !first_form.contains_key("client_secret"),
            "credentials went in the header"
        );

        expire_access_token(&pool, by_header.id).await;
        let mut calls = tokio::task::JoinSet::new();
        for _ in 0..8 {
            let usecase = usecase.clone();
            calls.spawn(async move { usecase.oauth_access_token(org_id, by_header.id).await });
        }
        let mut tokens = Vec::new();
        while let Some(joined) = calls.join_next().await {
            tokens.push(joined.unwrap().unwrap());
        }
        assert_eq!(
            behaviour.client_credential_calls.load(Ordering::SeqCst),
            2,
            "one renewal for eight callers"
        );
        assert!(tokens.iter().all(|token| token == "CC-2"), "{tokens:?}");

        let in_body = usecase
            .oauth_access_token(org_id, by_body.id)
            .await
            .unwrap();
        assert!(in_body.starts_with("CC-"));
        let last_form = behaviour.forms.lock().unwrap().last().unwrap().clone();
        assert_eq!(last_form["client_id"], "svc");
        assert_eq!(last_form["client_secret"], "svc-secret");
        match open_http_auth(&usecase, org_id, by_header.id)
            .await
            .unwrap()
        {
            HttpAuth::Bearer { token } => assert_eq!(token, "CC-2"),
            other => panic!("client credentials apply as a bearer: {other:?}"),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn only_a_kind_with_an_account_to_connect_can_start_an_authorization() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-kind").await;
        let usecase = use_case(pool.clone());
        let bearer = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "bearer_token".to_owned(),
                name: "Jeton".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(json!({ "token": "abc" })),
            })
            .await
            .unwrap()
            .0;

        let started = usecase
            .start_oauth_authorization(start_command(org_id, bearer.id, member))
            .await;
        let disconnected = usecase
            .disconnect_credential_account(org_id, bearer.id)
            .await;

        for outcome in [started.map(|_| ()), disconnected.map(|_| ())] {
            let Err(CoreError::Conflict(message)) = outcome else {
                panic!("a bearer token has no account to connect");
            };
            assert!(
                CredentialError::validation_message(&message).is_some(),
                "{message}"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn pending_authorizations_are_capped_per_credential() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-cap").await;
        let server = provider(Arc::new(Behaviour::default())).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;
        for _ in 0..MAX_PENDING_AUTHORIZATIONS_PER_CREDENTIAL {
            usecase
                .start_oauth_authorization(start_command(org_id, credential.id, member))
                .await
                .unwrap();
        }

        let excess = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await;

        assert!(matches!(excess, Err(CoreError::Conflict(_))), "{excess:?}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_token_url_is_filtered_by_the_https_rule_and_the_address_policy() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-guard").await;
        let behaviour = Arc::new(Behaviour::default());
        let server = provider(behaviour.clone()).await;
        let open = use_case(pool.clone());
        let guarded = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new())
            .with_cipher(Some(Arc::new(SecretCipher::new(&[7u8; 32]).unwrap())));
        let plain_http = code_credential(&open, org_id, &server, "Compte http").await;
        let loopback_https = open
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "oauth2_authorization_code".to_owned(),
                name: "Compte https".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(json!({
                    "authorization_url": "https://auth.example.com/authorize",
                    "token_url": server.url.replace("http://", "https://"),
                    "client_id": "mestier-client",
                    "client_secret": "client-secret-value",
                })),
            })
            .await
            .unwrap()
            .0;

        for credential in [&plain_http, &loopback_https] {
            let url = guarded
                .start_oauth_authorization(start_command(org_id, credential.id, member))
                .await
                .unwrap();

            let outcome = guarded
                .complete_oauth_authorization(callback(&state_of(&url), "good-code"))
                .await
                .unwrap();

            assert!(!outcome.connected, "{}", credential.name);
        }
        assert_eq!(behaviour.code_calls.load(Ordering::SeqCst), 0);
        assert!(server.requests().is_empty(), "nothing reached the server");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_oauth1_credential_connects_through_the_three_legs_and_signs_afterwards() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth1-flow").await;
        let seen: Arc<Mutex<Vec<(String, String)>>> = Arc::default();
        let recorded = seen.clone();
        let server = TestServer::start(move |request| {
            let authorization = request.headers.get("authorization").cloned().unwrap_or_default();
            recorded
                .lock()
                .unwrap()
                .push((request.path.clone(), authorization));
            match request.path.as_str() {
                "/request_token" => StubResponse::text(
                    200,
                    "oauth_token=REQ-TOKEN&oauth_token_secret=REQ-SECRET&oauth_callback_confirmed=true",
                ),
                "/access_token" => StubResponse::text(
                    200,
                    "oauth_token=ACCESS-TOKEN&oauth_token_secret=ACCESS-SECRET",
                ),
                _ => StubResponse::status(404),
            }
        })
        .await;
        let usecase = use_case(pool.clone());
        let credential = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "oauth1".to_owned(),
                name: "Compte OAuth1".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(json!({
                    "consumer_key": "ck",
                    "consumer_secret": "cs",
                    "request_token_url": format!("{}/request_token", server.url),
                    "authorization_url": format!("{}/authorize", server.url),
                    "access_token_url": format!("{}/access_token", server.url),
                })),
            })
            .await
            .unwrap()
            .0;
        assert_eq!(credential.status, CredentialStatus::NotConnected);

        let url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();

        assert_eq!(param_of(&url, "oauth_token").as_deref(), Some("REQ-TOKEN"));
        let first = seen.lock().unwrap()[0].1.clone();
        assert!(first.starts_with("OAuth "), "{first}");
        assert!(first.contains("oauth_callback="), "{first}");
        assert!(first.contains("oauth_signature="), "{first}");

        let wrong_token = usecase
            .complete_oauth_authorization(OauthCallback {
                state: "unknown".to_owned(),
                oauth_token: Some("REQ-TOKEN".to_owned()),
                oauth_verifier: Some("VERIFIER".to_owned()),
                ..OauthCallback::default()
            })
            .await;
        assert!(matches!(wrong_token, Err(CoreError::NotFound)));

        let state = {
            let row = sqlx::query!(
                "SELECT count(*) AS \"count!\" FROM automation.oauth_state WHERE credential_id = $1",
                credential.id
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(row.count, 1);
            let callback_url = first
                .split("oauth_callback=\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .unwrap()
                .replace("%3A", ":")
                .replace("%2F", "/")
                .replace("%3F", "?")
                .replace("%3D", "=");
            callback_url.split("state=").nth(1).unwrap().to_owned()
        };

        let mismatched = usecase
            .complete_oauth_authorization(OauthCallback {
                state: state.clone(),
                oauth_token: Some("SOMEONE-ELSES-TOKEN".to_owned()),
                oauth_verifier: Some("VERIFIER".to_owned()),
                ..OauthCallback::default()
            })
            .await
            .unwrap();
        assert!(
            !mismatched.connected,
            "the request token must be the one issued"
        );

        let url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();
        assert_eq!(param_of(&url, "oauth_token").as_deref(), Some("REQ-TOKEN"));
        let state = sqlx::query!(
            "SELECT state_hash FROM automation.oauth_state WHERE credential_id = $1",
            credential.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(state.state_hash.len(), 32);
        let issued = seen.lock().unwrap().last().unwrap().1.clone();
        let issued_state = issued
            .split("oauth_callback=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap()
            .replace("%3A", ":")
            .replace("%2F", "/")
            .replace("%3F", "?")
            .replace("%3D", "=")
            .split("state=")
            .nth(1)
            .unwrap()
            .to_owned();

        let outcome = usecase
            .complete_oauth_authorization(OauthCallback {
                state: issued_state,
                oauth_token: Some("REQ-TOKEN".to_owned()),
                oauth_verifier: Some("VERIFIER".to_owned()),
                ..OauthCallback::default()
            })
            .await
            .unwrap();

        assert!(outcome.connected);
        let exchanged = seen.lock().unwrap().last().unwrap().clone();
        assert_eq!(exchanged.0, "/access_token");
        assert!(
            exchanged.1.contains("oauth_verifier=\"VERIFIER\""),
            "{}",
            exchanged.1
        );
        assert!(
            exchanged.1.contains("oauth_token=\"REQ-TOKEN\""),
            "{}",
            exchanged.1
        );
        let connected = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(connected.status, CredentialStatus::Valid);
        assert_eq!(connected.expires_at, None);
        match open_http_auth(&usecase, org_id, credential.id)
            .await
            .unwrap()
        {
            HttpAuth::OAuth1(auth) => {
                assert_eq!(auth.token, "ACCESS-TOKEN");
                assert_eq!(auth.token_secret, "ACCESS-SECRET");
                assert_eq!(auth.consumer_key, "ck");
            }
            other => panic!("an OAuth1 credential signs requests: {other:?}"),
        }
    }

    #[test]
    fn an_outcome_holding_a_token_never_prints_it() {
        let token = TokenOutcome::Token("SECRET-ACCESS-TOKEN".to_owned());
        let failed = TokenOutcome::Failed("the provider said no".to_owned());

        for rendered in [format!("{token:?}"), format!("{token:#?}")] {
            assert!(!rendered.contains("SECRET-ACCESS-TOKEN"), "{rendered}");
            assert!(rendered.contains("[REDACTED]"), "{rendered}");
        }
        assert!(format!("{failed:?}").contains("the provider said no"));
    }

    async fn stored_expiry_after(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Duration {
        usecase
            .find_credential(org_id, id)
            .await
            .unwrap()
            .unwrap()
            .expires_at
            .expect("an expiry was recorded")
            - Utc::now()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_absurd_expires_in_is_capped_to_a_year_on_the_callback_the_refresh_and_client_credentials()
     {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-absurd").await;
        let behaviour = Arc::new(Behaviour::default());
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let code = code_credential(&usecase, org_id, &server, "Compte").await;
        let service = client_credentials_credential(&usecase, org_id, &server, "header").await;
        let a_year = Duration::days(365);

        for given in [
            json!(9_000_000_000_000_u64),
            json!(u64::MAX),
            json!("9000000000000"),
            json!("18446744073709551615"),
        ] {
            *behaviour.expires_in.lock().unwrap() = Some(given.clone());

            connect(&usecase, org_id, member, &code).await;
            let on_callback = stored_expiry_after(&usecase, org_id, code.id).await;
            expire_access_token(&pool, code.id).await;
            usecase.oauth_access_token(org_id, code.id).await.unwrap();
            let on_refresh = stored_expiry_after(&usecase, org_id, code.id).await;
            expire_access_token(&pool, service.id).await;
            usecase
                .oauth_access_token(org_id, service.id)
                .await
                .unwrap();
            let on_fetch = stored_expiry_after(&usecase, org_id, service.id).await;

            for (path, left) in [
                ("callback", on_callback),
                ("refresh", on_refresh),
                ("client credentials", on_fetch),
            ] {
                assert!(
                    left > a_year - Duration::minutes(5) && left <= a_year + Duration::minutes(5),
                    "{path} with {given}: {left}"
                );
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn twelve_callbacks_against_a_slow_provider_overlap_beyond_the_pool_size_because_none_holds_a_connection()
     {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-pool").await;
        let behaviour = Arc::new(Behaviour::default());
        behaviour.code_delay_ms.store(400, Ordering::SeqCst);
        let server = provider(behaviour.clone()).await;
        let small = PgPoolOptions::new()
            .max_connections(3)
            .acquire_timeout(StdDuration::from_secs(5))
            .connect_with(pool.connect_options().as_ref().clone())
            .await
            .unwrap();
        let usecase = use_case(small.clone());
        let mut states = Vec::new();
        for index in 0..12 {
            let credential =
                code_credential(&usecase, org_id, &server, &format!("Compte {index}")).await;
            let url = usecase
                .start_oauth_authorization(start_command(org_id, credential.id, member))
                .await
                .unwrap();
            states.push((credential.id, state_of(&url)));
        }

        let mut callbacks = tokio::task::JoinSet::new();
        for (_, state) in &states {
            let usecase = usecase.clone();
            let state = state.clone();
            callbacks.spawn(async move {
                usecase
                    .complete_oauth_authorization(callback(&state, "good-code"))
                    .await
            });
        }
        while let Some(joined) = callbacks.join_next().await {
            assert!(joined.unwrap().unwrap().connected);
        }

        assert_eq!(behaviour.code_calls.load(Ordering::SeqCst), 12);
        let most = behaviour.most_in_flight.load(Ordering::SeqCst);
        assert!(
            (4..=8).contains(&most),
            "more provider calls overlap than the pool has connections (3), never more than eight: {most}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn callbacks_for_one_credential_reach_the_provider_one_at_a_time() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-flight").await;
        let behaviour = Arc::new(Behaviour::default());
        behaviour.code_delay_ms.store(150, Ordering::SeqCst);
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;
        let mut states = Vec::new();
        for _ in 0..4 {
            let url = usecase
                .start_oauth_authorization(start_command(org_id, credential.id, member))
                .await
                .unwrap();
            states.push(state_of(&url));
        }

        let mut callbacks = tokio::task::JoinSet::new();
        for state in states {
            let usecase = usecase.clone();
            callbacks.spawn(async move {
                usecase
                    .complete_oauth_authorization(callback(&state, "good-code"))
                    .await
            });
        }
        while let Some(joined) = callbacks.join_next().await {
            assert!(joined.unwrap().unwrap().connected);
        }

        assert_eq!(behaviour.code_calls.load(Ordering::SeqCst), 4);
        assert_eq!(behaviour.most_in_flight.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn starting_an_oauth1_authorization_holds_no_connection_during_the_provider_call() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth1-pool").await;
        let behaviour = Arc::new(Behaviour::default());
        *behaviour.pool.lock().unwrap() = Some(pool.clone());
        let sampled = behaviour.clone();
        let server = TestServer::start(move |_| {
            sampled.linger(200);
            StubResponse::text(200, "oauth_token=REQ-TOKEN&oauth_token_secret=REQ-SECRET")
        })
        .await;
        let usecase = use_case(pool.clone());
        let credential = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: "oauth1".to_owned(),
                name: "Compte OAuth1".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(json!({
                    "consumer_key": "ck",
                    "consumer_secret": "cs",
                    "request_token_url": format!("{}/request_token", server.url),
                    "authorization_url": format!("{}/authorize", server.url),
                    "access_token_url": format!("{}/access_token", server.url),
                })),
            })
            .await
            .unwrap()
            .0;

        usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();

        assert_eq!(
            behaviour.connections_in_use.lock().unwrap().clone(),
            vec![0],
            "no transaction is open while the request token is fetched"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn moving_a_provider_address_needs_the_secret_again_and_disconnects_the_account() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-move").await;
        let server = provider(Arc::new(Behaviour::default())).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;
        connect(&usecase, org_id, member, &credential).await;
        let patch = |data: Value| UpdateCredentialCommand {
            org_id,
            id: credential.id,
            name: None,
            description: None,
            data: Some(data),
            expires_at: None,
        };

        let unrelated = usecase
            .update_credential(patch(json!({ "scopes": "openid email" })))
            .await
            .unwrap();
        assert_eq!(unrelated.status, CredentialStatus::Valid);
        assert_eq!(unrelated.account.as_deref(), Some("ada@example.com"));

        let without_secret = usecase
            .update_credential(patch(json!({ "token_url": "https://evil.example/token" })))
            .await
            .unwrap_err();
        let CoreError::Conflict(message) = without_secret else {
            panic!("expected a validation conflict");
        };
        assert!(
            CredentialError::validation_message(&message)
                .is_some_and(|text| text.contains("Client secret")),
            "{message}"
        );
        let untouched = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(untouched.status, CredentialStatus::Valid);
        assert_eq!(
            untouched.data["token_url"],
            json!(format!("{}/token", server.url))
        );

        let moved = usecase
            .update_credential(patch(json!({
                "token_url": "https://auth.example.com/token",
                "client_secret": "client-secret-value",
            })))
            .await
            .unwrap();

        assert_eq!(moved.status, CredentialStatus::NotConnected);
        assert_eq!(moved.account, None);
        assert_eq!(moved.secrets_set, vec!["client_secret".to_owned()]);
        let sealed = sealed_secrets(&pool, &usecase, credential.id).await;
        assert!(
            !sealed.contains_key("access_token") && !sealed.contains_key("refresh_token"),
            "{sealed:?}"
        );
        let refused = usecase
            .oauth_access_token(org_id, credential.id)
            .await
            .unwrap_err();
        assert!(refused.contains("not connected"), "{refused}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_callback_finishing_after_the_provider_address_moved_connects_nothing() {
        let pool = automation_pool().await;
        let (org_id, member) = seed_organization(&pool, "oauth-moved-meanwhile").await;
        let behaviour = Arc::new(Behaviour::default());
        behaviour.code_delay_ms.store(500, Ordering::SeqCst);
        let server = provider(behaviour.clone()).await;
        let usecase = use_case(pool.clone());
        let credential = code_credential(&usecase, org_id, &server, "Compte").await;
        let url = usecase
            .start_oauth_authorization(start_command(org_id, credential.id, member))
            .await
            .unwrap();
        let state = state_of(&url);

        let completing = tokio::spawn({
            let usecase = usecase.clone();
            async move {
                usecase
                    .complete_oauth_authorization(callback(&state, "good-code"))
                    .await
            }
        });
        tokio::time::sleep(StdDuration::from_millis(150)).await;
        usecase
            .update_credential(UpdateCredentialCommand {
                org_id,
                id: credential.id,
                name: None,
                description: None,
                data: Some(json!({
                    "token_url": "https://auth.example.com/token",
                    "client_secret": "client-secret-value",
                })),
                expires_at: None,
            })
            .await
            .unwrap();
        let outcome = completing.await.unwrap().unwrap();

        assert!(!outcome.connected);
        let after = usecase
            .find_credential(org_id, credential.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(after.status, CredentialStatus::NotConnected);
        let sealed = sealed_secrets(&pool, &usecase, credential.id).await;
        assert!(!sealed.contains_key("access_token"));
    }
}
