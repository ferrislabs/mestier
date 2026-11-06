use chrono::{DateTime, Utc};
use mestier_core::{
    Credential, CredentialField, CredentialKind, CredentialOrigin, CredentialStatus, OrganizationId,
};
use serde::Serialize;
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStatusResponse {
    Valid,
    Expired,
    NotConnected,
}

impl From<CredentialStatus> for CredentialStatusResponse {
    fn from(value: CredentialStatus) -> Self {
        match value {
            CredentialStatus::Valid => Self::Valid,
            CredentialStatus::Expired => Self::Expired,
            CredentialStatus::NotConnected => Self::NotConnected,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CredentialResponse {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub kind: String,
    pub name: String,
    pub description: String,
    pub status: CredentialStatusResponse,
    pub account: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    #[schema(value_type = Object)]
    pub data: Map<String, Value>,
    pub secrets_set: Vec<String>,
    pub used_by_workflows: i64,
    pub origin: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CredentialResponse {
    pub fn new(credential: Credential, used_by_workflows: i64, now: DateTime<Utc>) -> Self {
        let status = credential.status_at(now);
        Self {
            id: credential.id,
            organization_id: credential.org_id,
            kind: credential.kind,
            name: credential.name,
            description: credential.description,
            status: status.into(),
            account: credential.account,
            expires_at: credential.expires_at,
            data: credential.data,
            secrets_set: credential.secrets_set,
            used_by_workflows,
            origin: credential.origin.as_str().to_owned(),
            created_at: credential.created_at,
            updated_at: credential.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CredentialWithSecretResponse {
    #[serde(flatten)]
    pub credential: CredentialResponse,
    pub secret: Value,
}

pub fn secret_value(origin: CredentialOrigin, plaintext: &[u8]) -> Value {
    match origin {
        CredentialOrigin::Supplied => serde_json::from_slice(plaintext).unwrap_or(Value::Null),
        CredentialOrigin::Generated => {
            use base64::Engine;
            Value::String(base64::engine::general_purpose::STANDARD.encode(plaintext))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CredentialOptionResponse {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CredentialShowWhenResponse {
    pub field: String,
    pub one_of: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CredentialFieldResponse {
    pub name: String,
    pub label: String,
    pub kind: String,
    pub secret: bool,
    pub required: bool,
    pub default: Option<String>,
    pub placeholder: Option<String>,
    pub hint: Option<String>,
    pub options: Vec<CredentialOptionResponse>,
    pub show_when: Option<CredentialShowWhenResponse>,
}

impl From<&CredentialField> for CredentialFieldResponse {
    fn from(field: &CredentialField) -> Self {
        Self {
            name: field.name.to_owned(),
            label: field.label.to_owned(),
            kind: field.kind.code().to_owned(),
            secret: field.secret,
            required: field.required,
            default: field.default.map(str::to_owned),
            placeholder: field.placeholder.map(str::to_owned),
            hint: field.hint.map(str::to_owned),
            options: field
                .options
                .iter()
                .map(|option| CredentialOptionResponse {
                    value: option.value.to_owned(),
                    label: option.label.to_owned(),
                })
                .collect(),
            show_when: field.show_when.map(|rule| CredentialShowWhenResponse {
                field: rule.field.to_owned(),
                one_of: rule
                    .one_of
                    .iter()
                    .map(|value| (*value).to_owned())
                    .collect(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CredentialUsageResponse {
    pub usage: String,
    pub label: String,
    pub has_node: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CredentialKindResponse {
    pub kind: String,
    pub label: String,
    pub category: String,
    pub category_label: String,
    pub description: String,
    pub usages: Vec<CredentialUsageResponse>,
    pub origin_choices: Vec<String>,
    pub fields: Vec<CredentialFieldResponse>,
    pub supported: bool,
    pub executable: bool,
    pub connectable: bool,
}

impl From<&CredentialKind> for CredentialKindResponse {
    fn from(kind: &CredentialKind) -> Self {
        Self {
            kind: kind.kind.to_owned(),
            label: kind.label.to_owned(),
            category: kind.category.code().to_owned(),
            category_label: kind.category.label().to_owned(),
            description: kind.description.to_owned(),
            usages: kind
                .usages
                .iter()
                .map(|usage| CredentialUsageResponse {
                    usage: usage.code().to_owned(),
                    label: usage.label().to_owned(),
                    has_node: usage.has_node(),
                })
                .collect(),
            origin_choices: kind
                .origin_choices
                .iter()
                .map(|choice| (*choice).to_owned())
                .collect(),
            fields: kind
                .fields
                .iter()
                .map(CredentialFieldResponse::from)
                .collect(),
            supported: true,
            executable: kind.executable,
            connectable: kind.connectable,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use mestier_core::credential_kinds;
    use serde_json::json;

    use super::*;

    fn credential(kind: &str) -> Credential {
        let now = Utc::now();
        Credential {
            id: Uuid::from_u128(1),
            org_id: OrganizationId(Uuid::from_u128(2)),
            kind: kind.to_owned(),
            name: "Mon API".to_owned(),
            description: "Pour les devis".to_owned(),
            origin: CredentialOrigin::Supplied,
            status: CredentialStatus::Valid,
            account: None,
            expires_at: None,
            data: json!({ "username": "alice" }).as_object().cloned().unwrap(),
            secrets_set: vec!["password".to_owned()],
            fields_split: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn a_credential_response_carries_readable_values_and_secret_names_only() {
        let response = CredentialResponse::new(credential("http_basic"), 3, Utc::now());
        let wire = serde_json::to_value(&response).unwrap();

        assert_eq!(wire["data"], json!({ "username": "alice" }));
        assert_eq!(wire["secrets_set"], json!(["password"]));
        assert_eq!(wire["used_by_workflows"], json!(3));
        assert_eq!(wire["status"], json!("valid"));
        assert!(!wire.to_string().contains("p4ss"));
        let mut keys: Vec<&str> = wire
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "account",
                "created_at",
                "data",
                "description",
                "expires_at",
                "id",
                "kind",
                "name",
                "organization_id",
                "origin",
                "secrets_set",
                "status",
                "updated_at",
                "used_by_workflows",
            ]
        );
    }

    #[test]
    fn a_credential_past_its_expiry_is_listed_as_expired() {
        let now = Utc::now();
        let mut expired = credential("http_basic");
        expired.expires_at = Some(now - Duration::days(1));

        let wire = serde_json::to_value(CredentialResponse::new(expired, 0, now)).unwrap();

        assert_eq!(wire["status"], json!("expired"));
    }

    #[test]
    fn an_oauth_credential_without_tokens_is_listed_as_not_connected() {
        let mut oauth = credential("oauth2_authorization_code");
        oauth.status = CredentialStatus::NotConnected;

        let wire = serde_json::to_value(CredentialResponse::new(oauth, 0, Utc::now())).unwrap();

        assert_eq!(wire["status"], json!("not_connected"));
    }

    #[test]
    fn a_supplied_secret_reconstructs_the_original_json_object() {
        let data = json!({ "token": "abc123" });
        let plaintext = serde_json::to_vec(&data).unwrap();

        assert_eq!(secret_value(CredentialOrigin::Supplied, &plaintext), data);
    }

    #[test]
    fn a_generated_secret_is_shown_as_base64() {
        use base64::Engine;
        let plaintext = vec![1u8, 2, 3, 4, 5];

        let secret = secret_value(CredentialOrigin::Generated, &plaintext);

        let expected = base64::engine::general_purpose::STANDARD.encode(&plaintext);
        assert_eq!(secret, Value::String(expected));
    }

    #[test]
    fn the_kind_catalogue_serializes_with_french_labels_and_hints() {
        let kinds: Vec<CredentialKindResponse> = credential_kinds()
            .iter()
            .map(CredentialKindResponse::from)
            .collect();
        let smtp = kinds.iter().find(|kind| kind.kind == "smtp").unwrap();
        let wire = serde_json::to_value(smtp).unwrap();

        assert_eq!(wire["label"], json!("SMTP"));
        assert_eq!(wire["category"], json!("email"));
        assert_eq!(wire["category_label"], json!("E-mail"));
        assert_eq!(
            wire["description"],
            json!("Envoi d’e-mails depuis votre serveur de messagerie.")
        );
        assert_eq!(
            wire["usages"],
            json!([{ "usage": "smtp", "label": "E-mail SMTP", "has_node": false }])
        );
        assert_eq!(wire["fields"][1]["name"], json!("port"));
        assert_eq!(wire["fields"][1]["kind"], json!("number"));
        assert_eq!(wire["fields"][1]["default"], json!("465"));
        assert_eq!(wire["fields"][0]["placeholder"], json!("smtp.exemple.fr"));
        assert_eq!(wire["executable"], json!(true));
        assert_eq!(wire["supported"], json!(true));
    }

    #[test]
    fn a_kind_without_an_executor_says_so_and_a_conditional_field_carries_its_rule() {
        let ssh = credential_kinds()
            .iter()
            .find(|kind| kind.kind == "ssh")
            .unwrap();
        let wire = serde_json::to_value(CredentialKindResponse::from(ssh)).unwrap();

        assert_eq!(wire["executable"], json!(false));
        assert_eq!(
            wire["supported"],
            json!(true),
            "every kind can be created; `executable` says whether a node can use it yet"
        );
        let password = wire["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|field| field["name"] == "password")
            .unwrap();
        assert_eq!(
            password["show_when"],
            json!({ "field": "method", "one_of": ["password"] })
        );
        assert_eq!(password["secret"], json!(true));
        assert_eq!(password["kind"], json!("password"));
    }

    #[test]
    fn a_secret_field_never_carries_a_default() {
        for kind in credential_kinds() {
            for field in kind.fields.iter().filter(|field| field.secret) {
                assert_eq!(field.default, None, "{}.{}", kind.kind, field.name);
            }
        }
    }
}
