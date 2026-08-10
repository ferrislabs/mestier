//! Credentials: reusable authentication an organization configures once and
//! attaches to connectors.
//!
//! Separate from the connector on purpose. One Odoo account serves several
//! connectors, is entered once, and is revoked in one place.
//!
//! A credential is **not** the webhook signing secret. That one is generated
//! by Mestier so a receiver can verify us, and lives on the connector. A
//! credential is supplied by the user so Mestier can authenticate to them —
//! the opposite direction of trust, and it must never be "regenerated".
//!
//! A credential is an instance of a static [`AuthScheme`](super::connector::AuthScheme):
//! that module owns the field shapes (#197), this module owns the stored
//! instance, its origin, and its validation against those shapes.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use uuid::Uuid;

use super::connector::auth_scheme;

/// Where a credential's bytes came from, and therefore what happens when it
/// is rotated.
///
/// `Supplied`: entered by the user (an Odoo password, an API key). Mestier
/// never invents a replacement for it, so rotating one is refused — it is
/// replaced with a fresh `update` instead.
///
/// `Generated`: fabricated by Mestier itself (an outgoing signature, the same
/// idea as the webhook signing secret). Nothing about it comes from the
/// user, so rotating one just means generating new bytes.
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

    /// Only a `Generated` credential can be rotated: a `Supplied` one has
    /// nothing of Mestier's own to regenerate, only the user's own secret,
    /// which rotating would silently invalidate.
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

/// A stored credential. Its data is absent by design: sealed in the database
/// and never read back by the API, only by the worker that uses it (#202).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credential {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub kind: String,
    pub name: String,
    pub origin: CredentialOrigin,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fields for a new credential.
///
/// Both origins go through this: for `Supplied`, `data` carries what the
/// user typed in, checked against the scheme before it is sealed. For
/// `Generated`, Mestier fills the secret in itself and `data` is ignored.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateCredentialCommand {
    pub org_id: OrganizationId,
    pub kind: String,
    pub name: String,
    pub origin: CredentialOrigin,
    pub data: Option<serde_json::Value>,
}

/// Fields for an update. `data: None` renames without touching the sealed
/// bytes; `Some` replaces them after re-validating against the scheme.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateCredentialCommand {
    pub org_id: OrganizationId,
    pub id: Uuid,
    pub name: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CredentialError {
    #[error("unknown auth scheme `{kind}`")]
    UnknownScheme { kind: String },
    #[error("credential data must be a JSON object")]
    NotAnObject,
    #[error("missing field `{field}`")]
    MissingField { field: String },
    #[error("unknown field `{field}`")]
    UnknownField { field: String },
    #[error("a supplied credential is replaced, never rotated")]
    NotRotatable,
}

impl From<CredentialError> for CoreError {
    fn from(error: CredentialError) -> Self {
        CoreError::Conflict(error.to_string())
    }
}

/// Checks credential data against its auth scheme before it is sealed.
///
/// Both directions matter. A missing field cannot authenticate. An unknown
/// one is a typo that would otherwise be stored happily and only surface
/// much later, as an authentication failure against the third party, far
/// from where the mistake was made.
pub fn validate_credential_data(
    kind: &str,
    data: &serde_json::Value,
) -> Result<(), CredentialError> {
    let scheme = auth_scheme(kind).ok_or_else(|| CredentialError::UnknownScheme {
        kind: kind.to_owned(),
    })?;

    let object = data.as_object().ok_or(CredentialError::NotAnObject)?;

    for field in scheme.fields {
        if !object.contains_key(field.name) {
            return Err(CredentialError::MissingField {
                field: field.name.to_owned(),
            });
        }
    }

    for key in object.keys() {
        if !scheme.fields.iter().any(|field| field.name == key.as_str()) {
            return Err(CredentialError::UnknownField { field: key.clone() });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_carrying_every_required_field_is_accepted() {
        let data = serde_json::json!({
            "base_url": "https://odoo.example.com",
            "database": "prod",
            "username": "bot",
            "api_key": "secret",
        });

        assert!(validate_credential_data("odoo_api", &data).is_ok());
    }

    #[test]
    fn an_unknown_kind_is_refused_and_named() {
        let error = validate_credential_data("not_a_real_kind", &serde_json::json!({}))
            .expect_err("an unknown kind has no scheme to validate against");

        assert_eq!(
            error,
            CredentialError::UnknownScheme {
                kind: "not_a_real_kind".to_owned()
            }
        );
    }

    #[test]
    fn data_that_is_not_an_object_is_refused() {
        let error = validate_credential_data("bearer_token", &serde_json::json!("just a string"))
            .expect_err("a bare string carries no named fields");

        assert_eq!(error, CredentialError::NotAnObject);
    }

    #[test]
    fn a_missing_required_field_is_refused_and_named() {
        let data = serde_json::json!({
            "base_url": "https://odoo.example.com",
            "database": "prod",
            "username": "bot",
        });

        let error = validate_credential_data("odoo_api", &data)
            .expect_err("a credential missing its key cannot authenticate");

        assert_eq!(
            error,
            CredentialError::MissingField {
                field: "api_key".to_owned()
            }
        );
    }

    /// A typo would otherwise be stored and only surface as an authentication
    /// failure against the third party, far from where it was made.
    #[test]
    fn an_unknown_field_is_refused_and_named() {
        let data = serde_json::json!({
            "base_url": "https://odoo.example.com",
            "database": "prod",
            "username": "bot",
            "api_key": "secret",
            "aip_key": "typo",
        });

        let error = validate_credential_data("odoo_api", &data)
            .expect_err("an unknown field must not be silently stored");

        assert_eq!(
            error,
            CredentialError::UnknownField {
                field: "aip_key".to_owned()
            }
        );
    }

    #[test]
    fn a_generated_origin_can_be_rotated() {
        assert_eq!(CredentialOrigin::Generated.ensure_rotatable(), Ok(()));
    }

    #[test]
    fn a_supplied_origin_refuses_rotation() {
        assert_eq!(
            CredentialOrigin::Supplied.ensure_rotatable(),
            Err(CredentialError::NotRotatable)
        );
    }

    #[test]
    fn an_origin_round_trips_through_its_string_form() {
        assert_eq!(CredentialOrigin::Supplied.as_str(), "supplied");
        assert_eq!(CredentialOrigin::Generated.as_str(), "generated");
        assert_eq!(
            CredentialOrigin::from_str("supplied"),
            Ok(CredentialOrigin::Supplied)
        );
        assert_eq!(
            CredentialOrigin::from_str("generated"),
            Ok(CredentialOrigin::Generated)
        );
    }

    #[test]
    fn an_unknown_origin_string_is_rejected() {
        assert!(CredentialOrigin::from_str("not_an_origin").is_err());
    }

    #[test]
    fn a_credential_error_converts_to_a_conflict_naming_the_problem() {
        let error: CoreError = CredentialError::NotRotatable.into();

        assert!(matches!(error, CoreError::Conflict(_)));
        assert!(format!("{error}").contains("never rotated"));
    }
}
