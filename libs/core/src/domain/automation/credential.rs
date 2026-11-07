use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use serde_json::{Map, Value};
use uuid::Uuid;

use super::credential_kind::{
    CredentialField, CredentialFieldKind, CredentialKind, PROVIDER_URL_FIELDS, credential_kind,
    secret_guard,
};

pub const MAX_CREDENTIAL_DESCRIPTION_CHARS: usize = 500;

pub const MAX_CREDENTIAL_NAME_CHARS: usize = 120;

pub const MAX_TEXT_FIELD_BYTES: usize = 8 * 1024;

pub const MAX_LONG_FIELD_BYTES: usize = 64 * 1024;

pub const MAX_CREDENTIALS_PER_ORGANIZATION: i64 = 200;

pub const INTERNAL_SECRET_KEYS: &[&str] = &[
    "access_token",
    "refresh_token",
    "oauth_token",
    "oauth_token_secret",
];

const VALIDATION_MARKER: &str = "[validation] ";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialOrigin {
    Supplied,
    Generated,
}

impl CredentialOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supplied => "supplied",
            Self::Generated => "generated",
        }
    }

    pub fn ensure_rotatable(self) -> Result<(), CredentialError> {
        match self {
            Self::Generated => Ok(()),
            Self::Supplied => Err(CredentialError::NotRotatable),
        }
    }
}

impl std::fmt::Display for CredentialOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CredentialOrigin {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "supplied" => Ok(Self::Supplied),
            "generated" => Ok(Self::Generated),
            other => Err(format!("invalid credential origin `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialStatus {
    Valid,
    Expired,
    NotConnected,
}

impl CredentialStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Expired => "expired",
            Self::NotConnected => "not_connected",
        }
    }
}

impl FromStr for CredentialStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "valid" => Ok(Self::Valid),
            "expired" => Ok(Self::Expired),
            "not_connected" => Ok(Self::NotConnected),
            other => Err(format!("invalid credential status `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Credential {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub kind: String,
    pub name: String,
    pub description: String,
    pub origin: CredentialOrigin,
    pub status: CredentialStatus,
    pub account: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub data: Map<String, Value>,
    pub secrets_set: Vec<String>,
    pub fields_split: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Credential {
    pub fn initial_status(kind: &str) -> CredentialStatus {
        match credential_kind(kind) {
            Some(descriptor) if descriptor.connectable => CredentialStatus::NotConnected,
            _ => CredentialStatus::Valid,
        }
    }

    pub fn renews_itself(&self) -> bool {
        match self.kind.as_str() {
            "oauth2_client_credentials" => true,
            "oauth2_authorization_code" => self.holds_secret("refresh_token"),
            _ => false,
        }
    }

    pub fn holds_secret(&self, name: &str) -> bool {
        self.secrets_set.iter().any(|set| set == name)
    }

    pub fn status_at(&self, now: DateTime<Utc>) -> CredentialStatus {
        match self.status {
            CredentialStatus::NotConnected => CredentialStatus::NotConnected,
            CredentialStatus::Expired => CredentialStatus::Expired,
            CredentialStatus::Valid => {
                let lapsed = self.expires_at.is_some_and(|expiry| expiry <= now);
                if lapsed && !self.renews_itself() {
                    CredentialStatus::Expired
                } else {
                    CredentialStatus::Valid
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateCredentialCommand {
    pub org_id: OrganizationId,
    pub kind: String,
    pub name: String,
    pub description: Option<String>,
    pub origin: CredentialOrigin,
    pub data: Option<Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateCredentialCommand {
    pub org_id: OrganizationId,
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub data: Option<Value>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum CredentialError {
    #[error("Le type d’identification « {kind} » n’existe pas.")]
    UnknownKind { kind: String },
    #[error("Les paramètres de l’identification doivent former un objet JSON.")]
    NotAnObject,
    #[error("Le champ « {label} » est obligatoire.")]
    MissingField { label: String },
    #[error("Le champ « {field} » n’existe pas pour ce type d’identification.")]
    UnknownField { field: String },
    #[error(
        "Le champ « {label} » ne peut pas valoir null : envoyez une valeur, ou omettez-le pour conserver l’existante."
    )]
    NullValue { label: String },
    #[error("Le champ « {label} » doit être {expected}.")]
    WrongType {
        label: String,
        expected: &'static str,
    },
    #[error("La valeur du champ « {label} » ne fait pas partie des choix proposés.")]
    NotAnOption { label: String },
    #[error("Le champ « {label} » doit être compris entre 1 et 65 535.")]
    PortOutOfRange { label: String },
    #[error("Le champ « {label} » doit être une URL http ou https valide.")]
    InvalidUrl { label: String },
    #[error("JSON invalide : un objet est attendu.")]
    InvalidJsonObject,
    #[error("Définition JSON invalide : {reason}.")]
    InvalidCustomDefinition { reason: &'static str },
    #[error("Ce nom d’en-tête n’est pas valide.")]
    InvalidHeaderName,
    #[error("Ce nom d’en-tête est réservé et ne peut pas porter une identification.")]
    ReservedHeaderName,
    #[error("Les paramètres de l’identification sont obligatoires.")]
    MissingData,
    #[error(
        "Une identification générée par Mestier n’a pas de paramètres à modifier : régénérez-la."
    )]
    GeneratedHoldsNoFields,
    #[error("Ce type d’identification ne se connecte pas à un compte.")]
    NotConnectable,
    #[error("L’adresse de retour du fournisseur n’est pas acceptable.")]
    InvalidRedirectUri,
    #[error("L’adresse de retour dans l’application n’est pas acceptable.")]
    InvalidReturnTo,
    #[error(
        "Trop de connexions sont en attente pour cette identification : réessayez dans quelques minutes."
    )]
    TooManyAuthorizations,
    #[error("Le fournisseur n’a pas répondu comme attendu.")]
    ProviderUnavailable,
    #[error(
        "Ce type d’identification ne peut pas être généré par Mestier : saisissez ses paramètres."
    )]
    GeneratedNotAllowed,
    #[error("Modifier cette adresse exige de saisir de nouveau « {label} ».")]
    SecretRequired { label: String },
    #[error("Le champ « {label} » doit être une URL https.")]
    InsecureUrl { label: String },
    #[error("Une signature PLAINTEXT n’est acceptée qu’avec des URL https.")]
    PlaintextNeedsHttps,
    #[error("Le champ « {label} » ne peut pas dépasser {limit}.")]
    ValueTooLong { label: String, limit: &'static str },
    #[error("Le champ « {label} » contient un caractère interdit.")]
    ForbiddenCharacter { label: String },
    #[error("Cette organisation a atteint le nombre maximal d’identifications.")]
    TooManyCredentials,
    #[error("Donnez un nom à cette identification.")]
    BlankName,
    #[error("Le nom ne peut pas dépasser 120 caractères.")]
    NameTooLong,
    #[error("La description ne peut pas dépasser 500 caractères.")]
    DescriptionTooLong,
    #[error("Une identification fournie se remplace, elle ne se régénère pas.")]
    NotRotatable,
    #[error("Une identification de ce type porte déjà ce nom.")]
    NameTaken,
}

impl CredentialError {
    pub fn is_validation(&self) -> bool {
        !matches!(
            self,
            Self::NotRotatable
                | Self::NameTaken
                | Self::TooManyAuthorizations
                | Self::ProviderUnavailable
                | Self::TooManyCredentials
        )
    }

    pub fn validation_message(core_message: &str) -> Option<&str> {
        core_message.strip_prefix(VALIDATION_MARKER)
    }
}

impl From<CredentialError> for CoreError {
    fn from(error: CredentialError) -> Self {
        if error.is_validation() {
            CoreError::Conflict(format!("{VALIDATION_MARKER}{error}"))
        } else {
            CoreError::Conflict(error.to_string())
        }
    }
}

pub fn validate_name(name: &str) -> Result<String, CredentialError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CredentialError::BlankName);
    }
    if trimmed.contains('\0') {
        return Err(CredentialError::ForbiddenCharacter {
            label: "Nom".to_owned(),
        });
    }
    if trimmed.chars().count() > MAX_CREDENTIAL_NAME_CHARS {
        return Err(CredentialError::NameTooLong);
    }
    Ok(trimmed.to_owned())
}

pub fn validate_description(description: &str) -> Result<String, CredentialError> {
    let trimmed = description.trim();
    if trimmed.contains('\0') {
        return Err(CredentialError::ForbiddenCharacter {
            label: "Description".to_owned(),
        });
    }
    if trimmed.chars().count() > MAX_CREDENTIAL_DESCRIPTION_CHARS {
        return Err(CredentialError::DescriptionTooLong);
    }
    Ok(trimmed.to_owned())
}

pub fn ensure_origin_allowed(
    kind: &CredentialKind,
    origin: CredentialOrigin,
) -> Result<(), CredentialError> {
    match origin {
        CredentialOrigin::Supplied => Ok(()),
        CredentialOrigin::Generated if kind.origin_choices.contains(&"generated") => Ok(()),
        CredentialOrigin::Generated => Err(CredentialError::GeneratedNotAllowed),
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FieldSet {
    pub readable: Map<String, Value>,
    pub secrets: Map<String, Value>,
}

impl FieldSet {
    pub fn secret_names(&self) -> Vec<String> {
        self.secrets.keys().cloned().collect()
    }
}

pub fn is_internal_secret(name: &str) -> bool {
    INTERNAL_SECRET_KEYS.contains(&name)
}

pub fn assemble_fields(
    kind: &CredentialKind,
    stored: &Map<String, Value>,
    patch: &Value,
) -> Result<FieldSet, CredentialError> {
    let patch = patch.as_object().ok_or(CredentialError::NotAnObject)?;

    for (name, value) in patch {
        let Some(field) = kind.field(name) else {
            return Err(CredentialError::UnknownField {
                field: name.clone(),
            });
        };
        if value.is_null() {
            return Err(CredentialError::NullValue {
                label: field.label.to_owned(),
            });
        }
    }

    let mut merged = stored.clone();
    for (name, value) in patch {
        merged.insert(name.clone(), value.clone());
    }

    let mut set = FieldSet::default();
    for field in kind.fields {
        if !is_visible(kind, field, &merged) {
            continue;
        }
        let Some(value) = normalize(field, merged.get(field.name))? else {
            if field.required {
                return Err(CredentialError::MissingField {
                    label: field.label.to_owned(),
                });
            }
            continue;
        };
        check_rules(kind, field, &value)?;
        let target = if field.secret {
            &mut set.secrets
        } else {
            &mut set.readable
        };
        target.insert(field.name.to_owned(), value);
    }
    Ok(set)
}

fn effective_text(
    kind: &CredentialKind,
    name: &str,
    fields: &Map<String, Value>,
) -> Option<String> {
    fields
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            kind.field(name)
                .and_then(|field| field.default)
                .map(str::to_owned)
        })
}

pub fn destination_moved(
    kind: &CredentialKind,
    before: &Map<String, Value>,
    after: &Map<String, Value>,
) -> bool {
    secret_guard(kind.kind).is_some_and(|guard| {
        guard
            .destinations
            .iter()
            .any(|name| effective_text(kind, name, before) != effective_text(kind, name, after))
    })
}

pub fn require_secrets_after_a_move(
    kind: &CredentialKind,
    patch: &Value,
) -> Result<(), CredentialError> {
    let Some(guard) = secret_guard(kind.kind) else {
        return Ok(());
    };
    let supplied = patch.as_object();
    for secret in guard.secrets {
        if !supplied.is_some_and(|patch| patch.contains_key(*secret)) {
            return Err(CredentialError::SecretRequired {
                label: kind
                    .field(secret)
                    .map_or_else(|| (*secret).to_owned(), |field| field.label.to_owned()),
            });
        }
    }
    Ok(())
}

pub fn check_provider_urls(
    kind: &CredentialKind,
    readable: &Map<String, Value>,
    https_only: bool,
) -> Result<(), CredentialError> {
    if secret_guard(kind.kind).is_none() || kind.kind == "odoo_api" {
        return Ok(());
    }
    let plaintext = readable.get("signature_method").and_then(Value::as_str) == Some("PLAINTEXT");
    for name in PROVIDER_URL_FIELDS {
        let Some(url) = readable.get(*name).and_then(Value::as_str) else {
            continue;
        };
        if plaintext && !url.starts_with("https://") {
            return Err(CredentialError::PlaintextNeedsHttps);
        }
        if https_only && !url.starts_with("https://") {
            return Err(CredentialError::InsecureUrl {
                label: kind
                    .field(name)
                    .map_or_else(|| (*name).to_owned(), |field| field.label.to_owned()),
            });
        }
    }
    Ok(())
}

fn is_visible(kind: &CredentialKind, field: &CredentialField, merged: &Map<String, Value>) -> bool {
    let Some(rule) = field.show_when else {
        return true;
    };
    let controller = kind.field(rule.field);
    let current = merged
        .get(rule.field)
        .and_then(Value::as_str)
        .or_else(|| controller.and_then(|controller| controller.default))
        .unwrap_or_default();
    rule.one_of.contains(&current)
}

fn normalize(
    field: &CredentialField,
    value: Option<&Value>,
) -> Result<Option<Value>, CredentialError> {
    let wrong = |expected: &'static str| CredentialError::WrongType {
        label: field.label.to_owned(),
        expected,
    };
    let Some(value) = value else {
        return Ok(field.default_for_validation());
    };
    match field.kind {
        CredentialFieldKind::Text
        | CredentialFieldKind::Password
        | CredentialFieldKind::Textarea
        | CredentialFieldKind::Select => {
            let text = value.as_str().ok_or_else(|| wrong("un texte"))?;
            let text = if field.secret { text } else { text.trim() };
            if text.trim().is_empty() {
                return Ok(None);
            }
            check_size(field, text)?;
            if field.kind == CredentialFieldKind::Select
                && !field.options.iter().any(|option| option.value == text)
            {
                return Err(CredentialError::NotAnOption {
                    label: field.label.to_owned(),
                });
            }
            Ok(Some(Value::String(text.to_owned())))
        }
        CredentialFieldKind::Number => match value {
            Value::Number(number) => number
                .as_u64()
                .map(|number| Some(Value::from(number)))
                .ok_or_else(|| wrong("un nombre entier")),
            Value::String(text) if text.trim().is_empty() => Ok(None),
            Value::String(text) => text
                .trim()
                .parse::<u64>()
                .map(|number| Some(Value::from(number)))
                .map_err(|_| wrong("un nombre entier")),
            _ => Err(wrong("un nombre entier")),
        },
        CredentialFieldKind::Boolean => value
            .as_bool()
            .map(|flag| Some(Value::Bool(flag)))
            .ok_or_else(|| wrong("un booléen")),
        CredentialFieldKind::Json => {
            let parsed = match value {
                Value::Object(_) => value.clone(),
                Value::String(text) if text.trim().is_empty() => return Ok(None),
                Value::String(text) => {
                    check_size(field, text)?;
                    match serde_json::from_str::<Value>(text) {
                        Ok(parsed @ Value::Object(_)) => parsed,
                        _ => return Err(CredentialError::InvalidJsonObject),
                    }
                }
                _ => return Err(CredentialError::InvalidJsonObject),
            };
            if holds_nul(&parsed) {
                return Err(CredentialError::ForbiddenCharacter {
                    label: field.label.to_owned(),
                });
            }
            check_size(field, &parsed.to_string())?;
            Ok(Some(parsed))
        }
    }
}

fn holds_nul(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains('\0'),
        Value::Array(items) => items.iter().any(holds_nul),
        Value::Object(map) => map
            .iter()
            .any(|(name, member)| name.contains('\0') || holds_nul(member)),
        _ => false,
    }
}

fn check_size(field: &CredentialField, text: &str) -> Result<(), CredentialError> {
    if text.contains('\0') {
        return Err(CredentialError::ForbiddenCharacter {
            label: field.label.to_owned(),
        });
    }
    let (limit, label) = match field.kind {
        CredentialFieldKind::Textarea | CredentialFieldKind::Json => {
            (MAX_LONG_FIELD_BYTES, "64 Kio")
        }
        _ => (MAX_TEXT_FIELD_BYTES, "8 Kio"),
    };
    if text.len() > limit {
        return Err(CredentialError::ValueTooLong {
            label: field.label.to_owned(),
            limit: label,
        });
    }
    Ok(())
}

impl CredentialField {
    fn default_for_validation(&self) -> Option<Value> {
        match (self.kind, self.default) {
            (CredentialFieldKind::Select, Some(default)) => Some(Value::String(default.to_owned())),
            _ => None,
        }
    }
}

fn check_rules(
    kind: &CredentialKind,
    field: &CredentialField,
    value: &Value,
) -> Result<(), CredentialError> {
    if field.name == "port"
        && let Some(port) = value.as_u64()
        && !(1..=65_535).contains(&port)
    {
        return Err(CredentialError::PortOutOfRange {
            label: field.label.to_owned(),
        });
    }
    if field.name.ends_with("_url")
        && let Some(url) = value.as_str()
        && !is_http_url(url)
    {
        return Err(CredentialError::InvalidUrl {
            label: field.label.to_owned(),
        });
    }
    if field.name == "header_name"
        && let Some(name) = value.as_str()
    {
        check_header_name(name)?;
    }
    if kind.kind == "custom_auth" && field.name == "json" {
        check_custom_definition(value)?;
    }
    Ok(())
}

fn is_http_url(url: &str) -> bool {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    rest.is_some_and(|rest| !rest.is_empty() && !url.chars().any(char::is_whitespace))
}

fn check_header_name(name: &str) -> Result<(), CredentialError> {
    if !is_header_token(name) {
        return Err(CredentialError::InvalidHeaderName);
    }
    if is_reserved_header_name(name) {
        return Err(CredentialError::ReservedHeaderName);
    }
    Ok(())
}

fn is_header_token(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(character)
        })
}

fn check_custom_definition(value: &Value) -> Result<(), CredentialError> {
    let definition = value
        .as_object()
        .ok_or(CredentialError::InvalidJsonObject)?;
    for (section, content) in definition {
        let content = content
            .as_object()
            .ok_or(CredentialError::InvalidCustomDefinition {
                reason: "« headers », « qs » et « body » doivent être des objets",
            })?;
        match section.as_str() {
            "headers" => {
                for (name, header) in content {
                    check_header_name(name)?;
                    if !is_scalar(header) {
                        return Err(CredentialError::InvalidCustomDefinition {
                            reason: "les valeurs des en-têtes doivent être des textes, des nombres ou des booléens",
                        });
                    }
                }
            }
            "qs" => {
                for parameter in content.values() {
                    if !is_scalar(parameter) {
                        return Err(CredentialError::InvalidCustomDefinition {
                            reason: "les valeurs des paramètres doivent être des textes, des nombres ou des booléens",
                        });
                    }
                }
            }
            "body" => {}
            _ => {
                return Err(CredentialError::InvalidCustomDefinition {
                    reason: "seuls « headers », « qs » et « body » sont acceptés",
                });
            }
        }
    }
    if definition.is_empty() {
        return Err(CredentialError::InvalidCustomDefinition {
            reason: "la définition est vide",
        });
    }
    Ok(())
}

fn is_scalar(value: &Value) -> bool {
    matches!(value, Value::String(_) | Value::Number(_) | Value::Bool(_))
}

const RESERVED_HEADER_NAMES: [&str; 8] = [
    "host",
    "content-type",
    "content-length",
    "user-agent",
    "accept",
    "connection",
    "transfer-encoding",
    "cookie",
];

pub fn is_reserved_header_name(name: &str) -> bool {
    let name = name.trim().to_ascii_lowercase();
    RESERVED_HEADER_NAMES.contains(&name.as_str()) || name.starts_with("x-forwarded-")
}

#[cfg(test)]
impl Credential {
    pub fn fixture(org_id: OrganizationId, kind: &str, name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: common::generate_uuid_v7(),
            org_id,
            kind: kind.to_owned(),
            name: name.to_owned(),
            description: String::new(),
            origin: CredentialOrigin::Supplied,
            status: Credential::initial_status(kind),
            account: None,
            expires_at: None,
            data: Map::new(),
            secrets_set: Vec::new(),
            fields_split: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use serde_json::json;

    use super::super::credential_kind::credential_kinds;
    use super::*;

    fn kind(code: &str) -> &'static CredentialKind {
        credential_kind(code).unwrap_or_else(|| panic!("{code} is a known kind"))
    }

    fn create(code: &str, data: Value) -> Result<FieldSet, CredentialError> {
        assemble_fields(kind(code), &Map::new(), &data)
    }

    fn refused(code: &str, data: Value) -> CredentialError {
        create(code, data).expect_err("the data must be refused")
    }

    fn object(value: Value) -> Map<String, Value> {
        value.as_object().cloned().expect("an object")
    }

    #[test]
    fn data_carrying_every_required_field_is_accepted_and_split_in_readable_and_secret() {
        let set = create(
            "odoo_api",
            json!({
                "base_url": "https://odoo.example.com",
                "database": "prod",
                "username": "bot",
                "api_key": "secret",
            }),
        )
        .unwrap();

        assert_eq!(
            set.readable,
            object(json!({
                "base_url": "https://odoo.example.com",
                "database": "prod",
                "username": "bot",
            }))
        );
        assert_eq!(set.secrets, object(json!({ "api_key": "secret" })));
        assert_eq!(set.secret_names(), vec!["api_key".to_owned()]);
    }

    #[test]
    fn an_object_is_required() {
        assert_eq!(
            refused("bearer_token", json!("just a string")),
            CredentialError::NotAnObject
        );
        assert_eq!(
            refused("bearer_token", json!(["token"])),
            CredentialError::NotAnObject
        );
    }

    #[test]
    fn a_missing_required_field_is_named_by_its_french_label() {
        let error = refused(
            "odoo_api",
            json!({ "base_url": "https://odoo.example.com", "database": "prod", "username": "bot" }),
        );

        assert_eq!(
            error,
            CredentialError::MissingField {
                label: "Clé d’API".to_owned()
            }
        );
        assert_eq!(error.to_string(), "Le champ « Clé d’API » est obligatoire.");
    }

    #[test]
    fn a_blank_required_field_counts_as_missing() {
        for blank in [json!(""), json!("   "), json!("\t\n")] {
            assert_eq!(
                refused("bearer_token", json!({ "token": blank })),
                CredentialError::MissingField {
                    label: "Jeton".to_owned()
                }
            );
        }
    }

    #[test]
    fn an_unknown_field_is_refused_and_named() {
        let error = refused("bearer_token", json!({ "token": "t", "tokne": "typo" }));

        assert_eq!(
            error,
            CredentialError::UnknownField {
                field: "tokne".to_owned()
            }
        );
    }

    #[test]
    fn an_internal_token_name_cannot_be_written_by_a_client() {
        let error = refused(
            "oauth2_authorization_code",
            json!({ "access_token": "forged" }),
        );

        assert!(matches!(error, CredentialError::UnknownField { .. }));
    }

    #[test]
    fn an_explicit_null_is_refused_for_a_secret_and_for_a_readable_value() {
        assert_eq!(
            refused("http_basic", json!({ "username": "a", "password": null })),
            CredentialError::NullValue {
                label: "Mot de passe".to_owned()
            }
        );
        assert_eq!(
            refused("http_basic", json!({ "username": null, "password": "p" })),
            CredentialError::NullValue {
                label: "Utilisateur".to_owned()
            }
        );
    }

    #[test]
    fn a_wrong_type_is_refused_in_french() {
        let error = refused("bearer_token", json!({ "token": 12 }));

        assert_eq!(error.to_string(), "Le champ « Jeton » doit être un texte.");
        assert_eq!(
            refused("smtp", json!({ "host": "h", "port": "abc" })).to_string(),
            "Le champ « Port » doit être un nombre entier."
        );
        assert_eq!(
            refused("smtp", json!({ "host": "h", "port": 12.5 })).to_string(),
            "Le champ « Port » doit être un nombre entier."
        );
    }

    #[test]
    fn a_number_is_accepted_as_a_json_number_or_as_digits_and_stored_as_a_number() {
        for port in [json!(587), json!("587"), json!(" 587 ")] {
            let set = create("smtp", json!({ "host": "smtp.example.com", "port": port })).unwrap();

            assert_eq!(set.readable["port"], json!(587));
        }
    }

    #[test]
    fn a_port_outside_the_range_is_refused() {
        for port in [json!(0), json!(65_536), json!("70000")] {
            assert!(matches!(
                refused("smtp", json!({ "host": "h", "port": port })),
                CredentialError::PortOutOfRange { .. }
            ));
        }
    }

    #[test]
    fn a_select_takes_one_of_its_options_or_falls_back_to_its_default() {
        assert!(matches!(
            refused(
                "smtp",
                json!({ "host": "h", "port": 25, "security": "carrier-pigeon" })
            ),
            CredentialError::NotAnOption { .. }
        ));

        let set = create("smtp", json!({ "host": "h", "port": 25 })).unwrap();

        assert_eq!(set.readable["security"], json!("ssl"));
    }

    #[test]
    fn a_default_is_not_enough_for_a_required_field_without_one() {
        assert!(matches!(
            refused("smtp", json!({ "port": 25 })),
            CredentialError::MissingField { .. }
        ));
    }

    #[test]
    fn a_conditional_field_is_only_required_when_it_is_shown() {
        let with_key = create(
            "ssh",
            json!({
                "host": "h", "port": 22, "username": "root",
                "method": "private-key", "private_key": "-----BEGIN-----",
            }),
        )
        .unwrap();
        assert_eq!(with_key.secret_names(), vec!["private_key".to_owned()]);

        assert_eq!(
            refused(
                "ssh",
                json!({ "host": "h", "port": 22, "username": "root", "method": "password" })
            ),
            CredentialError::MissingField {
                label: "Mot de passe".to_owned()
            }
        );
        assert_eq!(
            refused(
                "ssh",
                json!({ "host": "h", "port": 22, "username": "root" })
            ),
            CredentialError::MissingField {
                label: "Clé privée".to_owned()
            }
        );
    }

    #[test]
    fn a_hidden_field_is_dropped_from_what_is_stored() {
        let set = create(
            "ssh",
            json!({
                "host": "h", "port": 22, "username": "root",
                "method": "password", "password": "p", "private_key": "stale",
            }),
        )
        .unwrap();

        assert_eq!(set.secrets, object(json!({ "password": "p" })));
    }

    #[test]
    fn an_optional_value_sent_empty_is_removed() {
        let set = create(
            "smtp",
            json!({ "host": "h", "port": 25, "username": "", "password": "" }),
        )
        .unwrap();

        assert!(!set.readable.contains_key("username"));
        assert!(set.secrets.is_empty());
    }

    #[test]
    fn a_patch_keeps_every_secret_it_does_not_mention_and_replaces_the_one_it_does() {
        let stored = object(json!({
            "host": "h", "port": 22, "username": "root", "method": "private-key",
            "private_key": "KEY", "passphrase": "PHRASE",
        }));

        let set = assemble_fields(kind("ssh"), &stored, &json!({ "passphrase": "NEW" })).unwrap();

        assert_eq!(
            set.secrets,
            object(json!({ "private_key": "KEY", "passphrase": "NEW" }))
        );
        assert_eq!(set.readable["host"], json!("h"));
    }

    #[test]
    fn a_patch_keeps_the_readable_values_it_does_not_mention() {
        let stored = object(json!({ "username": "alice", "password": "p" }));

        let set =
            assemble_fields(kind("http_basic"), &stored, &json!({ "password": "q" })).unwrap();

        assert_eq!(set.readable["username"], json!("alice"));
        assert_eq!(set.secrets["password"], json!("q"));
    }

    #[test]
    fn a_patch_can_clear_an_optional_secret_by_sending_it_empty() {
        let stored = object(json!({
            "host": "h", "port": 22, "username": "root", "method": "private-key",
            "private_key": "KEY", "passphrase": "PHRASE",
        }));

        let set = assemble_fields(kind("ssh"), &stored, &json!({ "passphrase": "" })).unwrap();

        assert_eq!(set.secrets, object(json!({ "private_key": "KEY" })));
    }

    #[test]
    fn a_patch_cannot_clear_a_required_secret() {
        let stored = object(json!({ "token": "t" }));

        assert_eq!(
            assemble_fields(kind("bearer_token"), &stored, &json!({ "token": "" })).unwrap_err(),
            CredentialError::MissingField {
                label: "Jeton".to_owned()
            }
        );
    }

    #[test]
    fn a_patch_with_an_explicit_null_secret_is_refused_and_changes_nothing() {
        let stored = object(json!({ "token": "t" }));

        assert!(matches!(
            assemble_fields(kind("bearer_token"), &stored, &json!({ "token": null })),
            Err(CredentialError::NullValue { .. })
        ));
    }

    #[test]
    fn the_header_credential_cannot_take_a_name_the_transport_owns() {
        for name in [
            "host",
            "Host",
            "CONTENT-TYPE",
            "content-length",
            "user-agent",
            "accept",
            "connection",
            "transfer-encoding",
            "cookie",
            "Cookie",
            "x-forwarded-for",
            "X-Forwarded-Proto",
        ] {
            assert_eq!(
                refused(
                    "http_header",
                    json!({ "header_name": name, "header_value": "v" })
                ),
                CredentialError::ReservedHeaderName,
                "{name}"
            );
        }
    }

    #[test]
    fn a_header_name_close_to_a_reserved_one_is_fine() {
        for name in [
            "X-Signature",
            "x-forwarded",
            "hostname",
            "accept-language",
            "x-cookie",
            "authorization",
            "x-api-key",
        ] {
            assert!(
                create(
                    "http_header",
                    json!({ "header_name": name, "header_value": "v" })
                )
                .is_ok(),
                "{name}"
            );
        }
    }

    #[test]
    fn a_header_name_with_a_space_or_a_colon_is_refused() {
        for name in ["X Api Key", "X-Key:", "clé"] {
            assert_eq!(
                refused(
                    "http_header",
                    json!({ "header_name": name, "header_value": "v" })
                ),
                CredentialError::InvalidHeaderName,
                "{name}"
            );
        }
    }

    #[test]
    fn the_hmac_header_follows_the_same_rules_and_the_algorithm_defaults_to_sha256() {
        let set = create(
            "hmac",
            json!({ "header_name": "X-Hub-Signature", "secret": "s" }),
        )
        .unwrap();

        assert_eq!(set.readable["algorithm"], json!("sha256"));
        assert_eq!(
            refused("hmac", json!({ "header_name": "Host", "secret": "s" })),
            CredentialError::ReservedHeaderName
        );
    }

    #[test]
    fn an_url_field_takes_an_http_or_https_url_only() {
        let base = |url: &str| {
            json!({
                "authorization_url": url, "token_url": "https://auth.example.com/token",
                "client_id": "id", "client_secret": "s",
            })
        };

        assert!(
            create(
                "oauth2_authorization_code",
                base("https://auth.example.com/authorize")
            )
            .is_ok()
        );
        for bad in [
            "ftp://x/y",
            "auth.example.com",
            "https://",
            "https://a b/c",
            "javascript:alert(1)",
        ] {
            assert!(
                matches!(
                    refused("oauth2_authorization_code", base(bad)),
                    CredentialError::InvalidUrl { .. }
                ),
                "{bad}"
            );
        }
    }

    #[test]
    fn the_custom_definition_takes_headers_qs_and_body_objects() {
        let set = create(
            "custom_auth",
            json!({ "json": { "headers": { "X-Token": "abc" }, "qs": { "k": 1 }, "body": { "a": [1] } } }),
        )
        .unwrap();

        assert!(set.readable.is_empty());
        assert_eq!(set.secrets["json"]["headers"], json!({ "X-Token": "abc" }));
    }

    #[test]
    fn the_custom_definition_can_arrive_as_json_text() {
        let set = create(
            "custom_auth",
            json!({ "json": "{ \"headers\": { \"X-Token\": \"abc\" } }" }),
        )
        .unwrap();

        assert_eq!(set.secrets["json"]["headers"]["X-Token"], json!("abc"));
    }

    #[test]
    fn the_custom_definition_refuses_what_is_not_a_json_object() {
        for bad in [
            json!("not json"),
            json!("[1]"),
            json!(12),
            json!(["x"]),
            json!("\"x\""),
        ] {
            assert_eq!(
                refused("custom_auth", json!({ "json": bad })),
                CredentialError::InvalidJsonObject
            );
        }
        assert_eq!(
            CredentialError::InvalidJsonObject.to_string(),
            "JSON invalide : un objet est attendu."
        );
    }

    #[test]
    fn the_custom_definition_refuses_unknown_sections_reserved_headers_and_nested_values() {
        assert!(matches!(
            refused("custom_auth", json!({ "json": { "cookies": {} } })),
            CredentialError::InvalidCustomDefinition { .. }
        ));
        assert!(matches!(
            refused("custom_auth", json!({ "json": {} })),
            CredentialError::InvalidCustomDefinition { .. }
        ));
        assert!(matches!(
            refused("custom_auth", json!({ "json": { "headers": ["x"] } })),
            CredentialError::InvalidCustomDefinition { .. }
        ));
        assert!(matches!(
            refused(
                "custom_auth",
                json!({ "json": { "headers": { "X": { "a": 1 } } } })
            ),
            CredentialError::InvalidCustomDefinition { .. }
        ));
        assert!(matches!(
            refused("custom_auth", json!({ "json": { "qs": { "x": [1] } } })),
            CredentialError::InvalidCustomDefinition { .. }
        ));
        assert_eq!(
            refused(
                "custom_auth",
                json!({ "json": { "headers": { "Host": "evil" } } })
            ),
            CredentialError::ReservedHeaderName
        );
        assert_eq!(
            refused(
                "custom_auth",
                json!({ "json": { "headers": { "X Bad": "v" } } })
            ),
            CredentialError::InvalidHeaderName
        );
    }

    #[test]
    fn smtp_needs_only_its_host_and_its_port() {
        assert!(create("smtp", json!({ "host": "h", "port": 25 })).is_ok());
    }

    #[test]
    fn every_kind_is_created_from_its_required_fields_alone() {
        let samples = [
            ("http_basic", json!({ "username": "a", "password": "p" })),
            ("bearer_token", json!({ "token": "t" })),
            (
                "http_header",
                json!({ "header_name": "X-K", "header_value": "v" }),
            ),
            (
                "api_key_query",
                json!({ "param_name": "key", "param_value": "v" }),
            ),
            ("digest", json!({ "username": "a", "password": "p" })),
            (
                "oauth2_authorization_code",
                json!({
                    "authorization_url": "https://a.example.com/auth",
                    "token_url": "https://a.example.com/token",
                    "client_id": "id", "client_secret": "s",
                }),
            ),
            (
                "oauth2_client_credentials",
                json!({ "token_url": "https://a.example.com/token", "client_id": "id", "client_secret": "s" }),
            ),
            (
                "oauth1",
                json!({
                    "consumer_key": "k", "consumer_secret": "s",
                    "request_token_url": "https://a.example.com/rt",
                    "authorization_url": "https://a.example.com/au",
                    "access_token_url": "https://a.example.com/at",
                }),
            ),
            ("jwt", json!({ "key": "shared-secret" })),
            (
                "mtls",
                json!({ "certificate": "-----BEGIN CERTIFICATE-----", "private_key": "-----BEGIN PRIVATE KEY-----" }),
            ),
            (
                "custom_auth",
                json!({ "json": { "headers": { "X-Token": "t" } } }),
            ),
            ("hmac", json!({ "header_name": "X-Sig", "secret": "s" })),
            ("smtp", json!({ "host": "smtp.example.com", "port": 465 })),
            (
                "imap",
                json!({ "host": "imap.example.com", "port": 993, "username": "u", "password": "p" }),
            ),
            (
                "ssh",
                json!({ "host": "h", "port": 22, "username": "u", "private_key": "KEY" }),
            ),
            (
                "ftp",
                json!({ "host": "h", "port": 22, "username": "u", "password": "p" }),
            ),
            (
                "sql",
                json!({ "host": "h", "port": 5432, "database": "d", "username": "u", "password": "p" }),
            ),
            (
                "odoo_api",
                json!({ "base_url": "https://o.example.com", "database": "d", "username": "u", "api_key": "k" }),
            ),
        ];

        assert_eq!(samples.len(), 18);
        for (code, data) in samples {
            assert!(create(code, data).is_ok(), "{code}");
        }
    }

    #[test]
    fn validation_errors_travel_as_a_marked_conflict_and_come_back_unmarked() {
        let error: CoreError = CredentialError::BlankName.into();
        let CoreError::Conflict(message) = error else {
            panic!("a credential error becomes a conflict");
        };

        assert_eq!(
            CredentialError::validation_message(&message),
            Some("Donnez un nom à cette identification.")
        );
    }

    #[test]
    fn the_name_and_rotation_conflicts_stay_plain_conflicts() {
        for error in [CredentialError::NameTaken, CredentialError::NotRotatable] {
            let CoreError::Conflict(message) = CoreError::from(error.clone()) else {
                panic!("a credential error becomes a conflict");
            };

            assert_eq!(CredentialError::validation_message(&message), None);
            assert_eq!(message, error.to_string());
        }
        assert_eq!(CredentialError::validation_message("slug taken"), None);
    }

    #[test]
    fn the_name_conflict_message_names_no_constraint() {
        let message = CredentialError::NameTaken.to_string();

        assert!(!message.contains("uq_"));
        assert!(!message.contains("automation"));
    }

    #[test]
    fn a_name_is_trimmed_and_cannot_be_blank() {
        assert_eq!(validate_name("  Mon API  "), Ok("Mon API".to_owned()));
        assert_eq!(validate_name("   "), Err(CredentialError::BlankName));
        assert_eq!(validate_name(""), Err(CredentialError::BlankName));
    }

    #[test]
    fn a_description_holds_five_hundred_characters_and_not_one_more() {
        assert!(validate_description(&"é".repeat(500)).is_ok());
        assert_eq!(
            validate_description(&"é".repeat(501)),
            Err(CredentialError::DescriptionTooLong)
        );
        assert_eq!(validate_description("  note  "), Ok("note".to_owned()));
        assert_eq!(validate_description(""), Ok(String::new()));
    }

    #[test]
    fn a_name_holds_a_hundred_and_twenty_characters_and_no_nul() {
        assert!(validate_name(&"é".repeat(120)).is_ok());
        assert_eq!(
            validate_name(&"é".repeat(121)),
            Err(CredentialError::NameTooLong)
        );
        assert!(matches!(
            validate_name("a\0b"),
            Err(CredentialError::ForbiddenCharacter { .. })
        ));
        assert!(matches!(
            validate_description("a\0b"),
            Err(CredentialError::ForbiddenCharacter { .. })
        ));
    }

    #[test]
    fn a_text_field_holds_eight_kibibytes_and_a_long_one_sixty_four() {
        let at_the_limit = "x".repeat(MAX_TEXT_FIELD_BYTES);
        let over = "x".repeat(MAX_TEXT_FIELD_BYTES + 1);

        assert!(create("bearer_token", json!({ "token": at_the_limit })).is_ok());
        assert!(matches!(
            refused("bearer_token", json!({ "token": over })),
            CredentialError::ValueTooLong { limit: "8 Kio", .. }
        ));
        assert!(matches!(
            refused("http_basic", json!({ "username": over, "password": "p" })),
            CredentialError::ValueTooLong { .. }
        ));

        let long = "x".repeat(MAX_LONG_FIELD_BYTES);
        assert!(create("jwt", json!({ "key": long })).is_ok());
        assert!(matches!(
            refused(
                "jwt",
                json!({ "key": "x".repeat(MAX_LONG_FIELD_BYTES + 1) })
            ),
            CredentialError::ValueTooLong {
                limit: "64 Kio",
                ..
            }
        ));
        let big_definition =
            json!({ "json": { "headers": { "X-T": "x".repeat(MAX_LONG_FIELD_BYTES) } } });
        assert!(matches!(
            refused("custom_auth", big_definition),
            CredentialError::ValueTooLong { .. }
        ));
    }

    #[test]
    fn a_nul_is_refused_in_readable_fields_secrets_and_json_definitions() {
        for (code, data) in [
            ("bearer_token", json!({ "token": "a\0b" })),
            ("http_basic", json!({ "username": "a\0b", "password": "p" })),
            ("jwt", json!({ "key": "a\0b" })),
            (
                "custom_auth",
                json!({ "json": { "headers": { "X-T": "a\0b" } } }),
            ),
            (
                "custom_auth",
                json!({ "json": "{\"headers\":{\"X-T\":\"a\\u0000b\"}}" }),
            ),
        ] {
            assert!(
                matches!(
                    refused(code, data.clone()),
                    CredentialError::ForbiddenCharacter { .. }
                ),
                "{code}: {data}"
            );
        }
    }

    #[test]
    fn only_a_kind_that_offers_it_can_be_generated() {
        for kind in credential_kinds() {
            let verdict = ensure_origin_allowed(kind, CredentialOrigin::Generated);
            if kind.origin_choices.contains(&"generated") {
                assert_eq!(verdict, Ok(()), "{}", kind.kind);
            } else {
                assert_eq!(
                    verdict,
                    Err(CredentialError::GeneratedNotAllowed),
                    "{}",
                    kind.kind
                );
            }
            assert_eq!(
                ensure_origin_allowed(kind, CredentialOrigin::Supplied),
                Ok(())
            );
        }
        assert!(CredentialError::GeneratedNotAllowed.is_validation());
    }

    #[test]
    fn moving_a_destination_is_seen_where_a_secret_travels_there() {
        let oauth = kind("oauth2_authorization_code");
        let before = object(json!({
            "authorization_url": "https://a/authorize",
            "token_url": "https://a/token",
            "client_id": "id",
            "scopes": "openid",
        }));
        let same_but_scopes = object(json!({
            "authorization_url": "https://a/authorize",
            "token_url": "https://a/token",
            "client_id": "other",
            "scopes": "profile",
        }));
        let moved_token = object(json!({
            "authorization_url": "https://a/authorize",
            "token_url": "https://evil/token",
        }));
        let added_userinfo = object(json!({
            "authorization_url": "https://a/authorize",
            "token_url": "https://a/token",
            "userinfo_url": "https://evil/me",
        }));

        assert!(!destination_moved(oauth, &before, &before));
        assert!(!destination_moved(oauth, &before, &same_but_scopes));
        assert!(destination_moved(oauth, &before, &moved_token));
        assert!(destination_moved(oauth, &before, &added_userinfo));
        assert!(!destination_moved(
            kind("bearer_token"),
            &before,
            &moved_token
        ));

        let one = kind("oauth1");
        let hmac =
            object(json!({ "request_token_url": "https://a/r", "signature_method": "HMAC-SHA1" }));
        let implicit = object(json!({ "request_token_url": "https://a/r" }));
        let plaintext =
            object(json!({ "request_token_url": "https://a/r", "signature_method": "PLAINTEXT" }));
        assert!(
            !destination_moved(one, &hmac, &implicit),
            "the default is the same method"
        );
        assert!(destination_moved(one, &hmac, &plaintext));
        assert!(destination_moved(
            kind("odoo_api"),
            &object(json!({ "base_url": "https://a" })),
            &object(json!({ "base_url": "https://b" }))
        ));
    }

    #[test]
    fn a_moved_destination_needs_its_secret_sent_again_and_says_which() {
        let oauth = kind("oauth2_authorization_code");

        let error = require_secrets_after_a_move(oauth, &json!({ "token_url": "https://b/t" }))
            .unwrap_err();

        assert_eq!(
            error,
            CredentialError::SecretRequired {
                label: "Client secret".to_owned()
            }
        );
        assert!(error.is_validation());
        assert_eq!(
            require_secrets_after_a_move(
                oauth,
                &json!({ "token_url": "https://b/t", "client_secret": "s" })
            ),
            Ok(())
        );
        assert!(matches!(
            require_secrets_after_a_move(kind("oauth1"), &json!({})),
            Err(CredentialError::SecretRequired { .. })
        ));
        assert_eq!(
            require_secrets_after_a_move(kind("bearer_token"), &json!({})),
            Ok(())
        );
    }

    #[test]
    fn provider_urls_are_https_when_the_instance_is_closed_and_plaintext_needs_https_everywhere() {
        let http = object(json!({ "authorization_url": "https://a/a", "token_url": "http://a/t" }));
        let https =
            object(json!({ "authorization_url": "https://a/a", "token_url": "https://a/t" }));
        let oauth = kind("oauth2_authorization_code");

        assert!(matches!(
            check_provider_urls(oauth, &http, true),
            Err(CredentialError::InsecureUrl { .. })
        ));
        assert_eq!(check_provider_urls(oauth, &http, false), Ok(()));
        assert_eq!(check_provider_urls(oauth, &https, true), Ok(()));
        assert_eq!(
            check_provider_urls(kind("bearer_token"), &http, true),
            Ok(())
        );

        let one = kind("oauth1");
        let plaintext_http = object(json!({
            "request_token_url": "http://a/r",
            "authorization_url": "https://a/a",
            "access_token_url": "https://a/t",
            "signature_method": "PLAINTEXT",
        }));
        let hmac_http = object(json!({
            "request_token_url": "http://a/r",
            "signature_method": "HMAC-SHA1",
        }));
        let plaintext_https = object(json!({
            "request_token_url": "https://a/r",
            "authorization_url": "https://a/a",
            "access_token_url": "https://a/t",
            "signature_method": "PLAINTEXT",
        }));
        assert_eq!(
            check_provider_urls(one, &plaintext_http, false),
            Err(CredentialError::PlaintextNeedsHttps)
        );
        assert_eq!(check_provider_urls(one, &hmac_http, false), Ok(()));
        assert_eq!(check_provider_urls(one, &plaintext_https, true), Ok(()));
    }

    #[test]
    fn a_generated_origin_can_be_rotated_and_a_supplied_one_cannot() {
        assert_eq!(CredentialOrigin::Generated.ensure_rotatable(), Ok(()));
        assert_eq!(
            CredentialOrigin::Supplied.ensure_rotatable(),
            Err(CredentialError::NotRotatable)
        );
    }

    #[test]
    fn an_origin_and_a_status_round_trip_through_their_string_form() {
        for origin in [CredentialOrigin::Supplied, CredentialOrigin::Generated] {
            assert_eq!(CredentialOrigin::from_str(origin.as_str()), Ok(origin));
        }
        for status in [
            CredentialStatus::Valid,
            CredentialStatus::Expired,
            CredentialStatus::NotConnected,
        ] {
            assert_eq!(CredentialStatus::from_str(status.as_str()), Ok(status));
        }
        assert!(CredentialOrigin::from_str("not_an_origin").is_err());
        assert!(CredentialStatus::from_str("not_a_status").is_err());
    }

    fn credential_of(kind: &str) -> Credential {
        Credential::fixture(OrganizationId(Uuid::from_u128(1)), kind, "Test")
    }

    #[test]
    fn a_credential_without_an_expiry_is_valid() {
        let credential = credential_of("bearer_token");

        assert_eq!(credential.status_at(Utc::now()), CredentialStatus::Valid);
    }

    #[test]
    fn a_credential_past_its_expiry_is_expired_and_one_before_it_is_valid() {
        let now = Utc::now();
        let mut credential = credential_of("bearer_token");

        credential.expires_at = Some(now - Duration::seconds(1));
        assert_eq!(credential.status_at(now), CredentialStatus::Expired);

        credential.expires_at = Some(now);
        assert_eq!(credential.status_at(now), CredentialStatus::Expired);

        credential.expires_at = Some(now + Duration::seconds(1));
        assert_eq!(credential.status_at(now), CredentialStatus::Valid);
    }

    #[test]
    fn an_oauth_kind_without_tokens_is_not_connected_whatever_the_clock_says() {
        let now = Utc::now();
        for code in ["oauth2_authorization_code", "oauth1"] {
            let mut credential = credential_of(code);
            credential.expires_at = Some(now - Duration::days(1));

            assert_eq!(credential.status, CredentialStatus::NotConnected);
            assert_eq!(credential.status_at(now), CredentialStatus::NotConnected);
        }
        assert_eq!(
            Credential::initial_status("oauth2_client_credentials"),
            CredentialStatus::Valid
        );
    }

    #[test]
    fn an_authorization_code_credential_past_its_expiry_is_expired_only_without_a_refresh_token() {
        let now = Utc::now();
        let mut credential = credential_of("oauth2_authorization_code");
        credential.status = CredentialStatus::Valid;
        credential.expires_at = Some(now - Duration::hours(1));

        assert_eq!(credential.status_at(now), CredentialStatus::Expired);

        credential.secrets_set = vec!["client_secret".to_owned(), "refresh_token".to_owned()];
        assert_eq!(credential.status_at(now), CredentialStatus::Valid);
    }

    #[test]
    fn a_client_credentials_token_past_its_expiry_renews_itself() {
        let now = Utc::now();
        let mut credential = credential_of("oauth2_client_credentials");
        credential.expires_at = Some(now - Duration::hours(1));

        assert_eq!(credential.status_at(now), CredentialStatus::Valid);
    }

    #[test]
    fn a_failed_refresh_is_expired_even_with_a_refresh_token_and_a_future_expiry() {
        let now = Utc::now();
        let mut credential = credential_of("oauth2_authorization_code");
        credential.status = CredentialStatus::Expired;
        credential.secrets_set = vec!["refresh_token".to_owned()];
        credential.expires_at = Some(now + Duration::hours(1));

        assert_eq!(credential.status_at(now), CredentialStatus::Expired);
    }

    #[test]
    fn no_descriptor_field_uses_a_name_the_oauth_flow_keeps_for_its_tokens() {
        for descriptor in super::super::credential_kind::credential_kinds() {
            for field in descriptor.fields {
                assert!(
                    !is_internal_secret(field.name),
                    "{}.{}",
                    descriptor.kind,
                    field.name
                );
            }
        }
    }
}
