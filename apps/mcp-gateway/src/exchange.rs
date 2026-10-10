use std::{
    collections::HashMap,
    future::Future,
    sync::Mutex,
    time::{Duration, Instant},
};

use reqwest::Client;
use serde::Deserialize;
use tracing::warn;

use crate::{config::ClientAuthMethod, verifier::Principal};

const SAFETY_MARGIN: Duration = Duration::from_secs(30);
const DEFAULT_LIFETIME: Duration = Duration::from_secs(60);
const GRANT_EXCHANGE: &str = "urn:ietf:params:oauth:grant-type:token-exchange";
const TOKEN_TYPE_ACCESS: &str = "urn:ietf:params:oauth:token-type:access_token";

#[derive(Debug, thiserror::Error)]
#[error("token exchange failed")]
pub struct ExchangeError;

#[derive(Debug, Clone)]
pub struct DelegatedToken {
    pub access_token: String,
    pub lifetime: Duration,
}

pub trait TokenExchanger {
    fn exchange(
        &self,
        principal: &Principal,
        subject_token: &str,
    ) -> impl Future<Output = Result<DelegatedToken, ExchangeError>> + Send;
}

type CacheKey = (String, String);

#[derive(Default)]
pub struct DelegatedTokenCache {
    entries: Mutex<HashMap<CacheKey, (String, Instant)>>,
}

impl DelegatedTokenCache {
    pub fn get(&self, sub: &str, client_id: &str, now: Instant) -> Option<String> {
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let (token, valid_until) = entries.get(&(sub.to_owned(), client_id.to_owned()))?;
        (now < *valid_until).then(|| token.clone())
    }

    pub fn insert(
        &self,
        sub: &str,
        client_id: &str,
        token: String,
        valid_until: Instant,
        now: Instant,
    ) {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries.retain(|_, (_, until)| now < *until);
        entries.insert((sub.to_owned(), client_id.to_owned()), (token, valid_until));
    }
}

pub struct CachedExchanger<X> {
    inner: X,
    cache: DelegatedTokenCache,
}

impl<X> CachedExchanger<X> {
    pub fn new(inner: X) -> Self {
        Self {
            inner,
            cache: DelegatedTokenCache::default(),
        }
    }
}

impl<X: TokenExchanger + Sync> TokenExchanger for CachedExchanger<X> {
    async fn exchange(
        &self,
        principal: &Principal,
        subject_token: &str,
    ) -> Result<DelegatedToken, ExchangeError> {
        let now = Instant::now();
        if let Some(token) = self.cache.get(&principal.sub, &principal.client_id, now) {
            return Ok(DelegatedToken {
                access_token: token,
                lifetime: Duration::ZERO,
            });
        }
        let fresh = self.inner.exchange(principal, subject_token).await?;
        let subject_left = Duration::from_secs(principal.exp.saturating_sub(unix_now()));
        let usable = fresh
            .lifetime
            .min(subject_left)
            .saturating_sub(SAFETY_MARGIN);
        if !usable.is_zero() {
            self.cache.insert(
                &principal.sub,
                &principal.client_id,
                fresh.access_token.clone(),
                now + usable,
                now,
            );
        }
        Ok(fresh)
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

pub struct FerrisKeyExchanger {
    http: Client,
    token_url: String,
    client_id: String,
    client_secret: String,
    audience: String,
    auth_method: ClientAuthMethod,
    actor: Mutex<Option<(String, Instant)>>,
}

impl FerrisKeyExchanger {
    pub fn new(
        http: Client,
        issuer: &str,
        client_id: String,
        client_secret: String,
        audience: String,
        auth_method: ClientAuthMethod,
    ) -> Self {
        Self {
            http,
            token_url: format!(
                "{}/protocol/openid-connect/token",
                issuer.trim_end_matches('/')
            ),
            client_id,
            client_secret,
            audience,
            auth_method,
            actor: Mutex::new(None),
        }
    }

    async fn post_token(
        &self,
        mut form: Vec<(&str, &str)>,
    ) -> Result<TokenResponse, ExchangeError> {
        let mut request = self.http.post(&self.token_url);
        match self.auth_method {
            ClientAuthMethod::Post => {
                form.push(("client_id", &self.client_id));
                form.push(("client_secret", &self.client_secret));
            }
            ClientAuthMethod::Basic => {
                request = request.basic_auth(&self.client_id, Some(&self.client_secret));
            }
        }
        let response = request
            .form(&form)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|error| {
                warn!(%error, "token endpoint call failed");
                ExchangeError
            })?;
        response.json().await.map_err(|error| {
            warn!(%error, "token endpoint answer unreadable");
            ExchangeError
        })
    }

    async fn actor_token(&self) -> Result<String, ExchangeError> {
        let now = Instant::now();
        if let Some((token, until)) = self
            .actor
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            && now < *until
        {
            return Ok(token.clone());
        }
        let response = self
            .post_token(vec![("grant_type", "client_credentials")])
            .await?;
        let lifetime = response
            .expires_in
            .map_or(DEFAULT_LIFETIME, Duration::from_secs)
            .saturating_sub(SAFETY_MARGIN);
        *self.actor.lock().unwrap_or_else(|e| e.into_inner()) =
            Some((response.access_token.clone(), now + lifetime));
        Ok(response.access_token)
    }
}

impl TokenExchanger for FerrisKeyExchanger {
    async fn exchange(
        &self,
        _principal: &Principal,
        subject_token: &str,
    ) -> Result<DelegatedToken, ExchangeError> {
        let actor_token = self.actor_token().await?;
        let response = self
            .post_token(vec![
                ("grant_type", GRANT_EXCHANGE),
                ("subject_token", subject_token),
                ("subject_token_type", TOKEN_TYPE_ACCESS),
                ("audience", &self.audience),
                ("actor_token", &actor_token),
                ("actor_token_type", TOKEN_TYPE_ACCESS),
            ])
            .await?;
        Ok(DelegatedToken {
            access_token: response.access_token,
            lifetime: response
                .expires_in
                .map_or(DEFAULT_LIFETIME, Duration::from_secs),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_serves_only_its_owner_until_it_expires() {
        let cache = DelegatedTokenCache::default();
        let now = Instant::now();
        cache.insert(
            "alice",
            "chatgpt",
            "t1".into(),
            now + Duration::from_secs(10),
            now,
        );

        assert_eq!(cache.get("alice", "chatgpt", now).as_deref(), Some("t1"));
        assert_eq!(cache.get("bob", "chatgpt", now), None);
        assert_eq!(cache.get("alice", "other", now), None);
        assert_eq!(
            cache.get("alice", "chatgpt", now + Duration::from_secs(10)),
            None
        );
    }

    #[test]
    fn inserting_purges_expired_entries() {
        let cache = DelegatedTokenCache::default();
        let now = Instant::now();
        cache.insert("a", "c", "t".into(), now + Duration::from_secs(1), now);
        let later = now + Duration::from_secs(5);
        cache.insert("b", "c", "t".into(), later + Duration::from_secs(5), later);

        assert_eq!(cache.entries.lock().unwrap().len(), 1);
    }
}
