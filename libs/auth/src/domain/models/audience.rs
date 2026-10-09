use crate::{Audience, Claims};

/// Which tokens the API takes. A token is for the API when its `aud` names
/// the API, or when it was issued to one of Mestier's own clients (the
/// webapp), whose tokens carry no API audience today.
///
/// Anything else is somebody else's token: a token an MCP client obtained
/// for the gateway must not reach the API directly, or the gateway's policy
/// and token exchange would be one request away from irrelevant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedAudience {
    api_audience: String,
    trusted_clients: Vec<String>,
}

impl AcceptedAudience {
    pub fn new(api_audience: impl Into<String>, trusted_clients: Vec<String>) -> Self {
        Self {
            api_audience: api_audience.into(),
            trusted_clients,
        }
    }

    pub fn accepts(&self, claims: &Claims) -> bool {
        let names_the_api = match &claims.aud {
            Some(Audience::Single(audience)) => *audience == self.api_audience,
            Some(Audience::Multiple(audiences)) => audiences.contains(&self.api_audience),
            None => false,
        };
        names_the_api
            || authorized_party(claims)
                .is_some_and(|client| self.trusted_clients.iter().any(|c| c == client))
    }
}

fn authorized_party(claims: &Claims) -> Option<&str> {
    claims
        .extra
        .get("azp")
        .and_then(serde_json::Value::as_str)
        .or(claims.client_id.as_deref())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn claims(value: serde_json::Value) -> Claims {
        let mut base = json!({ "sub": "user-1", "iss": "https://auth", "scope": "openid" });
        base.as_object_mut()
            .unwrap()
            .extend(value.as_object().unwrap().clone());
        serde_json::from_value(base).unwrap()
    }

    fn accepted() -> AcceptedAudience {
        AcceptedAudience::new("mestier-api", vec!["mestier".to_owned()])
    }

    #[test]
    fn a_token_naming_the_api_is_accepted() {
        assert!(accepted().accepts(&claims(json!({ "aud": "mestier-api" }))));
        assert!(accepted().accepts(&claims(json!({ "aud": ["account", "mestier-api"] }))));
    }

    #[test]
    fn a_token_of_a_trusted_client_is_accepted() {
        assert!(accepted().accepts(&claims(json!({ "aud": ["account"], "azp": "mestier" }))));
        assert!(accepted().accepts(&claims(json!({ "client_id": "mestier" }))));
    }

    #[test]
    fn a_token_for_someone_else_is_refused() {
        let for_the_gateway = claims(json!({
            "aud": "https://mcp.example/mcp",
            "azp": "https://chatgpt.example/client.json",
        }));
        assert!(!accepted().accepts(&for_the_gateway));
        assert!(!accepted().accepts(&claims(json!({}))));
    }
}
