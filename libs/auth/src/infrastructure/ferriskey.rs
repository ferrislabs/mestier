use std::sync::Arc;

use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{AuthError, Claims, Identity, domain::ports::AuthRepository};

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Clone)]
pub struct FerrisKeyRepository {
    pub http: Arc<Client>,
    pub issuer: String,
    pub audience: Option<String>,
}

impl FerrisKeyRepository {
    pub fn new(issuer: impl Into<String>, audience: Option<String>) -> Self {
        Self {
            http: Arc::new(Client::new()),
            issuer: issuer.into(),
            audience,
        }
    }

    #[tracing::instrument(skip(self), fields(issuer = %self.issuer), err)]
    async fn fetch_jwks(&self) -> Result<Jwks, AuthError> {
        let url = format!("{}/protocol/openid-connect/certs", self.issuer);

        let resp = self.http.get(url).send().await.map_err(|e| {
            warn!("failed to fetch jwks: {}", e);
            AuthError::Network {
                message: e.to_string(),
            }
        })?;

        if resp.status().is_client_error() || resp.status().is_server_error() {
            warn!("failed to fetch jwks: {}", resp.status());
            return Err(AuthError::Network {
                message: format!("failed to fetch jwks: {}", resp.status()),
            });
        }

        let bytes = resp.bytes().await.map_err(|e| {
            warn!("failed to read jwks response body: {}", e);
            AuthError::Network {
                message: e.to_string(),
            }
        })?;

        let jwks: Jwks = serde_json::from_slice(&bytes).map_err(|e| AuthError::Network {
            message: e.to_string(),
        })?;

        Ok(jwks)
    }
}

impl AuthRepository for FerrisKeyRepository {
    #[tracing::instrument(skip_all, err)]
    async fn identify(&self, token: &str) -> Result<Identity, AuthError> {
        let claims = self.validate_token(token).await?;

        Ok(Identity::from(claims))
    }

    #[tracing::instrument(skip_all, err)]
    async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let header = decode_header(token).map_err(|e| {
            warn!("failed to decode token header: {}", e);
            AuthError::InvalidToken {
                message: e.to_string(),
            }
        })?;

        let kid = header.kid.ok_or_else(|| {
            warn!("token is missing kid header");
            AuthError::InvalidToken {
                message: "missing kind".into(),
            }
        })?;

        let jwks = self.fetch_jwks().await?;

        info!("fetched jwks with {} keys", jwks.keys.len());

        let keys = jwks.keys;

        let key = keys
            .iter()
            .find(|k| k.kid == kid)
            .ok_or_else(|| AuthError::KeyNotFound { key: kid.clone() })?;

        let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|e| {
            warn!("failed to build decoding key from jwk: {}", e);
            AuthError::Internal {
                message: e.to_string(),
            }
        })?;

        let mut validation = Validation::new(Algorithm::RS256);

        validation.validate_aud = false;
        validation.validate_exp = false;

        let data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
            warn!("failed to decode token: {}", e);
            AuthError::InvalidToken {
                message: e.to_string(),
            }
        })?;

        let claims = data.claims;

        let now = Utc::now().timestamp();

        if claims.exp.unwrap_or(0) < now {
            warn!("token is expired");
            return Err(AuthError::Expired);
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use chrono::Utc;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

    use crate::{Audience, AuthError, Claims, Subject, domain::ports::AuthRepository};

    use super::FerrisKeyRepository;

    const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDHF23PRKIxgKZC
8b8NihRuCh/PTx8bdX+x5Mp6WR44eVFNirC/j9mhmtK5vezS3CokPgpl0g1CVfBp
iZR1OEZs+Y0cFFJBZTWxuDiUz3jAQIqQlN7WH6dNsOu14FJS799Tv4yAC/wtx7ig
yLpncaQYJE5CkSOAGd+P7YBT2ONtOs0dR0+bdTbDkpu1MlIEsVMojzJFVuGGKtJp
cgLrYVZKACNop5y84tQUJx7vLW2JfdEZleFJ6k4g9DNnl/Y6njLTsKVtCKakORf3
wiqbk80IQxN7labaVQlXd1GooBC+7mBxwyXbFW35eM31GgQBPVaqBOHpBLu60knW
t7hC0x/xAgMBAAECggEAHjaCfg1K1dtRn+Ai37GgJxDXQfUeYeLjZYI0bfu/N8/F
VFCjQPbaDom5x+E4IsmxhX16w3fsdjAng0STKHTJTzlRvjyhPPZYfydXQtH3X6mL
vaQx6umz0Hj0VE3+AEMRr5pmfnoTI3lnHdNIYnFe9yDvVW/EJOkIQcXHjzHfVZBt
ofFGHL8NjJ008VEVwDtscaCq+ibfoEghvI9GMffd/HqZAYd9qhrz+wiT8ZQAFbp5
kTlP6YBUJ+mo2K7OkNdGPivgaxQhijwqc9d53eFMrmnETxliAHN1Alniud16o1j8
TpaIwF0Y+y6trmHrKXWaQkVRbPfYT2QTSmpTeLe1jwKBgQDzEevZNdjBJWvjvqaX
5n5F3ZPQD67XKghgokkNa+uKrIvHrzG4HDXrR7R24SHBxTmHGgw2k3WRfaFBnoHN
n7BoJNK+M8ddP3b0ea2kFpPAkWuWmOxv0VQykt721vfkHohBu5ra5eoXXd4Efnj5
PqX50JCVPT+k5Xl4R9dpbniziwKBgQDRrp4QZoiX3GXEmddqIn5ZwMrY/ia9Z8M0
da3I/+PCUFw23HEP0T6LskS8g64dG63hhrCy0BZN+WrJQu/m82cAJaRsQCbzilIt
K6/3NtXlu4SmXotGxEpn26X03j0YO1osKLFgd2FiT/0KiIQYj1/Ipyst3YghCIjR
zYm1KKx58wKBgBAV4oa4UoTNpisnJb0tqrOS60I8l3RzuqQyeSUjPC4sJv/q7x5g
94x/bUjksygwlhMDvUUrUv9y0eYWyD5EUBdEQJIHuSzJk2SwXLZcLCD1Pqpzqkno
D2tdXtX0+eilwJyg/ql3x5sOQjAH8peD9tXmYHsP15NhAD3eeznl7qTrAoGBAIXj
8pqWXnJaEcHQWnUzQWseaGjXIPWg5E0DN805WL4jgj6l1Kw8+KtLUgjuLKf5nLZ9
wybrKNLxiPaq/3WBxyuY3b0h2b15fa/KTbqWEU94xeNWS6kMflaDMx2BK5HllFbO
RTVMBas5WGL5eSAVrRv7Yt8OrnYpdPRDQsOjDT9xAoGBAMq7pYVEJBWoyFYWDnSY
LoQgUrpiRssRjaCMHOpEBxjtOTv3TzeyzHWD7+r2+y/qToJXcdA8jEyhaSeUa7mr
9e2VtIC/6Ouhmfb0+mwgwO/zQHR0sd/ruyNc7v4FBgYfZ/XqvYtzzTZzhNmvX9gQ
HUim3t4M1KMtX1QmMKKCg4i4
-----END PRIVATE KEY-----"#;

    const TEST_KID: &str = "test-kid";
    const TEST_N: &str = "xxdtz0SiMYCmQvG_DYoUbgofz08fG3V_seTKelkeOHlRTYqwv4_ZoZrSub3s0twqJD4KZdINQlXwaYmUdThGbPmNHBRSQWU1sbg4lM94wECKkJTe1h-nTbDrteBSUu_fU7-MgAv8Lce4oMi6Z3GkGCROQpEjgBnfj-2AU9jjbTrNHUdPm3U2w5KbtTJSBLFTKI8yRVbhhirSaXIC62FWSgAjaKecvOLUFCce7y1tiX3RGZXhSepOIPQzZ5f2Op4y07ClbQimpDkX98Iqm5PNCEMTe5Wm2lUJV3dRqKAQvu5gccMl2xVt-XjN9RoEAT1WqgTh6QS7utJJ1re4QtMf8Q";
    const TEST_E: &str = "AQAB";

    fn build_token_with_header(header_json: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
        let payload = URL_SAFE_NO_PAD.encode(br#"{"sub":"u"}"#);

        // Signature content is irrelevant for tests that fail before signature validation.
        format!("{header}.{payload}.sig")
    }

    fn start_server_with_response(status: &str, body: &str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind tcp listener");
        let addr = listener.local_addr().expect("failed to read local addr");
        let status = status.to_string();
        let body = body.to_string();

        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0_u8; 2048];
                let _ = stream.read(&mut buf);

                let response = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        format!("http://{}", addr)
    }

    fn build_jwks_json(kid: &str, n: &str, e: &str) -> String {
        format!(r#"{{"keys":[{{"kid":"{kid}","n":"{n}","e":"{e}"}}]}}"#)
    }

    fn build_signed_token(exp: i64, client_id: Option<String>) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(TEST_KID.to_string());

        let claims = Claims {
            sub: Subject("user-123".to_string()),
            iss: "https://auth.ferriscord.com".to_string(),
            aud: Some(Audience::Single("ferriscord-api".to_string())),
            exp: Some(exp),
            email: Some("john.doe@example.com".to_string()),
            email_verified: Some(true),
            name: Some("John Doe".to_string()),
            preferred_username: Some("johndoe".to_string()),
            given_name: Some("John".to_string()),
            family_name: Some("Doe".to_string()),
            scope: "openid profile email".to_string(),
            client_id,
            extra: serde_json::Map::new(),
        };

        let key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes())
            .expect("failed to build rsa encoding key");

        encode(&header, &claims, &key).expect("failed to encode jwt")
    }

    #[tokio::test]
    async fn test_new_sets_issuer_and_audience() {
        let repo = FerrisKeyRepository::new(
            "https://auth.ferriscord.com",
            Some("ferriscord-api".to_string()),
        );

        assert_eq!(repo.issuer, "https://auth.ferriscord.com");
        assert_eq!(repo.audience, Some("ferriscord-api".to_string()));
    }

    #[tokio::test]
    async fn test_validate_token_returns_invalid_token_for_bad_format() {
        let repo = FerrisKeyRepository::new("http://localhost:65530", None);

        let err = repo
            .validate_token("not-a-jwt")
            .await
            .expect_err("expected invalid token error");

        assert!(matches!(err, AuthError::InvalidToken { .. }));
    }

    #[tokio::test]
    async fn test_validate_token_returns_invalid_token_when_kid_is_missing() {
        let repo = FerrisKeyRepository::new("http://localhost:65530", None);
        let token = build_token_with_header(r#"{"alg":"RS256","typ":"JWT"}"#);

        let err = repo
            .validate_token(&token)
            .await
            .expect_err("expected invalid token error");

        match err {
            AuthError::InvalidToken { message } => assert!(message.contains("missing kind")),
            other => panic!("expected invalid token error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_validate_token_returns_network_error_when_jwks_fails() {
        let repo = FerrisKeyRepository::new("http://no-such-issuer.invalid", None);
        let token = build_token_with_header(r#"{"alg":"RS256","typ":"JWT","kid":"test-kid"}"#);

        let err = repo
            .validate_token(&token)
            .await
            .expect_err("expected network error");

        match err {
            AuthError::Network { .. } => {}
            other => panic!("expected network error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_fetch_jwks_returns_network_error_for_http_status_error() {
        let issuer = start_server_with_response("500 Internal Server Error", r#"{"keys":[]}"#);
        let repo = FerrisKeyRepository::new(issuer, None);

        let err = repo.fetch_jwks().await.expect_err("expected network error");

        match err {
            AuthError::Network { message } => {
                assert!(message.contains("failed to fetch jwks: 500 Internal Server Error"));
            }
            other => panic!("expected network error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_fetch_jwks_returns_network_error_for_invalid_json() {
        let issuer = start_server_with_response("200 OK", "not-json");
        let repo = FerrisKeyRepository::new(issuer, None);

        let err = repo.fetch_jwks().await.expect_err("expected network error");

        assert!(matches!(err, AuthError::Network { .. }));
    }

    #[tokio::test]
    async fn test_validate_token_returns_key_not_found_when_kid_not_in_jwks() {
        let issuer =
            start_server_with_response("200 OK", &build_jwks_json("other-kid", TEST_N, TEST_E));
        let repo = FerrisKeyRepository::new(issuer, None);
        let token = build_token_with_header(r#"{"alg":"RS256","typ":"JWT","kid":"wanted-kid"}"#);

        let err = repo
            .validate_token(&token)
            .await
            .expect_err("expected key-not-found error");

        match err {
            AuthError::KeyNotFound { key } => assert_eq!(key, "wanted-kid"),
            other => panic!("expected key-not-found error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_validate_token_returns_internal_for_invalid_rsa_components() {
        let issuer =
            start_server_with_response("200 OK", &build_jwks_json(TEST_KID, "!!!", TEST_E));
        let repo = FerrisKeyRepository::new(issuer, None);
        let token = build_token_with_header(r#"{"alg":"RS256","typ":"JWT","kid":"test-kid"}"#);

        let err = repo
            .validate_token(&token)
            .await
            .expect_err("expected internal error");

        assert!(matches!(err, AuthError::Internal { .. }));
    }

    #[tokio::test]
    async fn test_validate_token_returns_expired_for_past_exp() {
        let issuer =
            start_server_with_response("200 OK", &build_jwks_json(TEST_KID, TEST_N, TEST_E));
        let repo = FerrisKeyRepository::new(issuer, None);
        let token = build_signed_token(Utc::now().timestamp() - 120, None);

        let err = repo
            .validate_token(&token)
            .await
            .expect_err("expected expired error");

        assert!(matches!(err, AuthError::Expired));
    }

    #[tokio::test]
    async fn test_validate_token_returns_claims_for_valid_token() {
        let issuer =
            start_server_with_response("200 OK", &build_jwks_json(TEST_KID, TEST_N, TEST_E));
        let repo = FerrisKeyRepository::new(issuer, None);
        let token = build_signed_token(Utc::now().timestamp() + 120, None);

        let claims = repo
            .validate_token(&token)
            .await
            .expect("expected valid claims");

        assert_eq!(claims.sub.0, "user-123");
        assert_eq!(claims.preferred_username, Some("johndoe".to_string()));
    }

    #[tokio::test]
    async fn test_identity_returns_identity_from_valid_token() {
        let issuer =
            start_server_with_response("200 OK", &build_jwks_json(TEST_KID, TEST_N, TEST_E));
        let repo = FerrisKeyRepository::new(issuer, None);
        let token = build_signed_token(
            Utc::now().timestamp() + 120,
            Some("ferriscord-bot".to_string()),
        );

        let identity = repo.identify(&token).await.expect("expected identity");

        assert!(identity.is_client());
        assert_eq!(identity.id(), "user-123");
        assert_eq!(identity.username(), "ferriscord-bot");
    }
}
