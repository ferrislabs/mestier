use base64::{Engine, engine::general_purpose};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    claims::{Claims, Jwt},
    errors::AuthError,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Token(pub String);

impl Token {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn decode_manual(&self) -> Result<Jwt, AuthError> {
        let mut parts = self.0.split('.');
        let (Some(_header), Some(payload), Some(_signature), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(AuthError::InvalidToken {
                message: "JWT must have 3 parts separated by dots".to_string(),
            });
        };

        let decoded = general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|e| {
                error!("failed to decode JWT payload: {:?}", e);
                AuthError::InvalidToken {
                    message: format!("failed to decode JWT payload: {:?}", e),
                }
            })?;

        let payload_str = String::from_utf8(decoded).map_err(|e| {
            error!("failed to decode JWT payload: {:?}", e);
            AuthError::InvalidToken {
                message: format!("Failed to convert payload to UTF-8: {}", e),
            }
        })?;

        let claims: Claims = serde_json::from_str(&payload_str).map_err(|e| {
            error!("failed to deserialize JWT claims: {:?}", e);
            AuthError::InvalidToken {
                message: format!("failed to deserialize JWT claims: {:?}", e),
            }
        })?;

        Ok(Jwt {
            claims,
            token: self.clone(),
        })
    }

    pub fn extract_claims(&self) -> Result<Claims, AuthError> {
        let jwt = self.decode_manual()?;
        Ok(jwt.claims)
    }
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde_json::json;

    use crate::{AuthError, Token};

    fn create_test_jwt(claims: serde_json::Value) -> String {
        let header = Header::default();
        let key = EncodingKey::from_secret("secret".as_ref());
        encode(&header, &claims, &key).unwrap()
    }

    #[test]
    fn test_token_new_and_as_str() {
        let token_str = "test.token.here";
        let token = Token::new(token_str);

        assert_eq!(token.as_str(), token_str);
    }

    #[test]
    fn test_token_decode_manual_success() {
        let test_claims = json!({
            "sub": "user-123",
            "iss": "https://auth.ferriscord.com",
            "aud": "ferriscord-api",
            "exp": 1735689600,
            "email": "john.doe@example.com",
            "email_verified": true,
            "scope": "read:messages write:messages",
            "preferred_username": "johndoe",
            "name": "John Doe"
        });

        let jwt_token = create_test_jwt(test_claims);
        let token = Token::new(jwt_token.clone());
        let result = token.decode_manual();

        assert!(result.is_ok());
        let jwt = result.unwrap();
        assert_eq!(jwt.claims.sub.0, "user-123");
        assert_eq!(jwt.claims.iss, "https://auth.ferriscord.com");
        assert_eq!(jwt.token.0, jwt_token);
    }

    #[test]
    fn test_token_extract_claims() {
        let test_claims = json!({
            "sub": "user-456",
            "iss": "https://auth.ferriscord.com",
            "email": "jane@example.com",
            "email_verified": true,
            "scope": "admin",
            "preferred_username": "jane"
        });

        let jwt_token = create_test_jwt(test_claims);
        let token = Token::new(jwt_token);
        let result = token.extract_claims();

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub.0, "user-456");
        assert_eq!(claims.email, Some("jane@example.com".to_string()));
    }

    #[test]
    fn test_token_decode_manual_invalid_format() {
        let token = Token::new("invalid.token".to_string());
        let result = token.decode_manual();

        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::InvalidToken { message } => {
                assert!(message.contains("3 parts"));
            }
            _ => panic!("Expected InvalidToken error"),
        }
    }

    #[test]
    fn test_token_decode_manual_with_real_token() {
        let token_str = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJiaE9ZRENETC14TFhyWVRGZERTMmlwMzdHdHhFNlpUVVI4a2swSm9CVDhzIn0.eyJleHAiOjE3NjExMTc5NTYsImlhdCI6MTc2MTExNzg5NiwianRpIjoib25ydHJvOjJhMjNjYjkyLTc1MTktYzgzYS0wMGM2LWIxNDQyOTlkYjE1NSIsImlzcyI6Imh0dHA6Ly9sb2NhbGhvc3Q6ODAwMC9yZWFsbXMvbWFzdGVyIiwiYXVkIjoiYWNjb3VudCIsInN1YiI6IjE0NDM0Y2JhLThmMzItNDliYi1hMzllLTgzNzhhN2NkZGVhMyIsInR5cCI6IkJlYXJlciIsImF6cCI6ImFwaSIsInNpZCI6ImY2YjUwZWY2LTJlNjItNjAxNS1lNTJjLTA5NzA4NWUyYTAxOCIsImFjciI6IjEiLCJhbGxvd2VkLW9yaWdpbnMiOlsiLyoiXSwicmVhbG1fYWNjZXNzIjp7InJvbGVzIjpbImRlZmF1bHQtcm9sZXMtbWFzdGVyIiwib2ZmbGluZV9hY2Nlc3MiLCJ1bWFfYXV0aG9yaXphdGlvbiJdfSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoicHJvZmlsZSBlbWFpbCIsImVtYWlsX3ZlcmlmaWVkIjp0cnVlLCJuYW1lIjoiTmF0aGFlbCBCb25uYWwiLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJuYXRoYWVsIiwiZ2l2ZW5fbmFtZSI6Ik5hdGhhZWwiLCJmYW1pbHlfbmFtZSI6IkJvbm5hbCIsImVtYWlsIjoibmF0aGFlbEBib25uYWwuY2xvdWQifQ.ApKQsnjT2gCgqngCndHTNU2W9YJzuHGHRLk4OE-_b4Sk650vSUS0AhMWPuAgEwVjLm2y8UpOJ_64BXDcnQMZzKHNo2_xj5c8P8glvBM-02YJlR_ssbUlReJPvLLKzwFTPdKF_FDsEIXkroV-ds8aU5OmOX8emdxb79XzdHkaWbl13IErHqMnRMsAvh742ZQeCqbedr8R3uH6V5qbbNu7H9kTf2EGX7G66rfpY-Zl8EyR4fWCVwjVLr_5tLsUFteajADf2RtW9dZRsUW9M9g9WIzT_tNdsTQhBj1q3kHkwhhC6hVVz2VaLNgYKikLu8QDfGy4BZ6nHZobrq4eKr3HQg";

        let token = Token::new(token_str.to_string());
        let result = token.decode_manual();

        assert!(result.is_ok());
        let jwt = result.unwrap();
        assert_eq!(jwt.claims.sub.0, "14434cba-8f32-49bb-a39e-8378a7cddea3");
        assert_eq!(jwt.claims.email, Some("nathael@bonnal.cloud".to_string()));
    }

    #[test]
    fn test_token_decode_manual_invalid_base64() {
        let token = Token::new("e30.invalid_base64.sig");
        let result = token.decode_manual();

        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::InvalidToken { message } => {
                assert!(message.contains("failed to decode JWT payload"));
            }
            _ => panic!("Expected InvalidToken error"),
        }
    }

    #[test]
    fn test_token_decode_manual_invalid_utf8() {
        let token = Token::new("e30._w.sig");
        let result = token.decode_manual();

        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::InvalidToken { message } => {
                assert!(message.contains("UTF-8"));
            }
            _ => panic!("Expected InvalidToken error"),
        }
    }

    #[test]
    fn test_token_decode_manual_invalid_json() {
        let token = Token::new("e30.bm90LWpzb24.sig");
        let result = token.decode_manual();

        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::InvalidToken { message } => {
                assert!(message.contains("failed to deserialize JWT claims"));
            }
            _ => panic!("Expected InvalidToken error"),
        }
    }
}
