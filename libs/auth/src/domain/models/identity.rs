use serde::{Deserialize, Serialize};

use crate::domain::models::{claims::Claims, client::Client, user::User};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Identity {
    User(User),
    Client(Client),
}

impl Identity {
    pub fn id(&self) -> &str {
        match self {
            Identity::User(u) => &u.id,
            Identity::Client(c) => &c.id,
        }
    }

    pub fn is_user(&self) -> bool {
        matches!(self, Identity::User(_))
    }

    pub fn is_client(&self) -> bool {
        matches!(self, Identity::Client(_))
    }

    pub fn username(&self) -> &str {
        match self {
            Identity::User(u) => &u.username,
            Identity::Client(c) => &c.client_id,
        }
    }

    pub fn roles(&self) -> &[String] {
        match self {
            Identity::User(u) => &u.roles,
            Identity::Client(c) => &c.roles,
        }
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles().iter().any(|r| r == role)
    }
}

impl From<Claims> for Identity {
    fn from(claims: Claims) -> Self {
        if let Some(client_id) = claims.client_id {
            Identity::Client(Client {
                id: claims.sub.0,
                client_id,
                roles: Vec::new(),
                scopes: Vec::new(),
            })
        } else {
            Identity::User(User {
                id: claims.sub.0.clone(),
                email: claims.email,
                name: claims.name,
                roles: Vec::new(),
                username: claims.preferred_username.unwrap_or(claims.sub.0),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::domain::models::{
        claims::{Audience, Claims},
        identity::Identity,
    };

    fn create_user_claims() -> Claims {
        Claims {
            sub: crate::domain::models::claims::Subject("user-123".to_string()),
            iss: "https://auth.ferriscord.com".to_string(),
            aud: Some(Audience::Single("ferriscord-api".to_string())),
            email: Some("john.doe@example.com".to_string()),
            email_verified: Some(true),
            exp: None,
            name: Some("John Doe".to_string()),
            preferred_username: Some("johndoe".to_string()),
            given_name: Some("John".to_string()),
            family_name: Some("Doe".to_string()),
            scope: "openid profile email".to_string(),
            client_id: None,
            extra: {
                let mut map = serde_json::Map::new();
                map.insert(
                    "realm_access".to_string(),
                    json!({
                        "roles": ["user", "moderator"]
                    }),
                );
                map
            },
        }
    }

    fn create_service_account_claims() -> Claims {
        Claims {
            sub: crate::domain::models::claims::Subject("service-123".to_string()),
            iss: "https://auth.ferriscord.com".to_string(),
            aud: Some(Audience::Single("ferriscord-api".to_string())),
            email: None,
            email_verified: Some(false),
            name: None,
            exp: None,
            preferred_username: Some("service-account-bot".to_string()),
            given_name: None,
            family_name: None,
            scope: "admin:all read:users write:messages".to_string(),
            client_id: Some("ferriscord-bot".to_string()),
            extra: {
                let mut map = serde_json::Map::new();
                map.insert(
                    "realm_access".to_string(),
                    json!({
                        "roles": ["service", "bot"]
                    }),
                );
                map
            },
        }
    }

    #[test]
    fn test_claims_to_identity_user() {
        let claims = create_user_claims();
        let identity: Identity = claims.into();

        match identity {
            Identity::User(user) => {
                assert_eq!(user.id, "user-123");
                assert_eq!(user.username, "johndoe");
                assert_eq!(user.email, Some("john.doe@example.com".to_string()));
                assert_eq!(user.name, Some("John Doe".to_string()));
            }
            Identity::Client(_) => panic!("Expected User, got Client"),
        }
    }

    #[test]
    fn test_claims_to_identity_service_account() {
        let claims = create_service_account_claims();
        let identity: Identity = claims.into();

        match identity {
            Identity::Client(client) => {
                assert_eq!(client.id, "service-123");
                assert_eq!(client.client_id, "ferriscord-bot");
            }
            Identity::User(_) => panic!("Expected Client, got User"),
        }
    }

    #[test]
    fn test_identity_accessors_for_user() {
        let claims = create_user_claims();
        let identity: Identity = claims.into();

        assert!(identity.is_user());
        assert!(!identity.is_client());
        assert_eq!(identity.id(), "user-123");
        assert_eq!(identity.username(), "johndoe");
        assert!(identity.roles().is_empty());
        assert!(!identity.has_role("admin"));
    }

    #[test]
    fn test_identity_accessors_for_client() {
        let claims = create_service_account_claims();
        let identity: Identity = claims.into();

        assert!(identity.is_client());
        assert!(!identity.is_user());
        assert_eq!(identity.id(), "service-123");
        assert_eq!(identity.username(), "ferriscord-bot");
        assert!(identity.roles().is_empty());
        assert!(!identity.has_role("service"));
    }
}
