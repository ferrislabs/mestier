use serde::{Deserialize, Serialize};

use crate::token::Token;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Role(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scope(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subject(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Audience {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Subject,
    pub iss: String,
    pub aud: Option<Audience>,
    pub exp: Option<i64>,

    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub preferred_username: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub scope: String,
    pub client_id: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Jwt {
    pub claims: Claims,
    pub token: Token,
}

#[cfg(test)]
mod tests {
    use crate::domain::models::claims::{Audience, Claims, Role, Scope, Subject};

    #[test]
    fn test_subject_deserialize_from_json() {
        let json = r#""user-123""#;
        let subject: Subject = serde_json::from_str(json).unwrap();

        assert_eq!(subject.0, "user-123");
    }

    #[test]
    fn test_role_deserialize_from_json() {
        let json = r#""admin""#;
        let role: Role = serde_json::from_str(json).unwrap();
        assert_eq!(role.0, "admin");
    }

    #[test]
    fn test_scope_deserialize_from_json() {
        let json = r#""read:users""#;
        let scope: Scope = serde_json::from_str(json).unwrap();
        assert_eq!(scope.0, "read:users");
    }

    #[test]
    fn test_claims_deserialize_basic() {
        let json = r#"{
            "exp": 1761117956,
            "iat": 1761117896,
            "jti": "onrtro:2a23cb92-7519-c83a-00c6-b144299db155",
            "iss": "http://localhost:8000/realms/master",
            "aud": "account",
            "sub": "14434cba-8f32-49bb-a39e-8378a7cddea3",
            "typ": "Bearer",
            "azp": "api",
            "sid": "f6b50ef6-2e62-6015-e52c-097085e2a018",
            "acr": "1",
            "allowed-origins": [
              "/*"
            ],
            "realm_access": {
              "roles": [
                "default-roles-master",
                "offline_access",
                "uma_authorization"
              ]
            },
            "resource_access": {
              "account": {
                "roles": [
                  "manage-account",
                  "manage-account-links",
                  "view-profile"
                ]
              }
            },
            "scope": "profile email",
            "email_verified": true,
            "name": "Nathael Bonnal",
            "preferred_username": "nathael",
            "given_name": "Nathael",
            "family_name": "Bonnal",
            "email": "nathael@bonnal.cloud"
        }"#;

        let claims: Claims = serde_json::from_str(json).unwrap();

        assert_eq!(claims.sub.0, "14434cba-8f32-49bb-a39e-8378a7cddea3");
        assert_eq!(claims.iss, "http://localhost:8000/realms/master");
        assert_eq!(claims.preferred_username, Some("nathael".to_string()));
    }

    #[test]
    fn test_claims_with_extra_fields() {
        let json = r#"{
            "sub": "user-456",
            "iss": "https://auth.ferriscord.com",
            "exp": 1735689600,
            "email": null,
            "scope": "openid connect",
            "preferred_username": null,
            "email_verified": true,
            "name": "John Doe",
            "custom_field": "custom_value",
            "nested": {
                "data": "test"
            }
        }"#;

        let claims: Claims = serde_json::from_str(json).unwrap();

        assert_eq!(claims.sub.0, "user-456");
        assert_eq!(claims.email, None);
        assert_eq!(claims.preferred_username, None);

        assert_eq!(
            claims.extra.get("custom_field").unwrap().as_str().unwrap(),
            "custom_value"
        );
        assert!(claims.extra.contains_key("nested"));
    }

    #[test]
    fn test_claims_deserialize_with_array_audience_and_null_client_id() {
        let json = r#"{
            "sub": "019c3a7d-30e8-7474-af67-65ff8186bfb6",
            "iat": 1770516273,
            "jti": "c5e66041-e6c0-4be1-8129-174f5e60422f",
            "iss": "http://localhost:3333/realms/mestier",
            "typ": "Bearer",
            "azp": "console",
            "aud": [
              "mestier-realm",
              "account"
            ],
            "scope": "address email offline_access openid phone profile",
            "exp": 1770516573,
            "preferred_username": "nathaelb",
            "email": "pro.nathaelbonnal@gmail.com",
            "client_id": null
        }"#;

        let claims: Claims = serde_json::from_str(json).unwrap();

        assert_eq!(claims.sub.0, "019c3a7d-30e8-7474-af67-65ff8186bfb6");
        assert_eq!(claims.preferred_username, Some("nathaelb".to_string()));
        assert_eq!(
            claims.email,
            Some("pro.nathaelbonnal@gmail.com".to_string())
        );
        assert!(claims.client_id.is_none());
        assert_eq!(
            claims.aud,
            Some(Audience::Multiple(vec![
                "mestier-realm".to_string(),
                "account".to_string()
            ]))
        );
    }
}
