use serde::{Deserialize, Serialize};

use crate::domain::types::Properties;

/// Well-known subject types used by Mestier. The AuthZen spec leaves the
/// `type` field free-form, so this enum is purely a convenience: it
/// serializes to the canonical strings and accepts any other value via
/// [`SubjectKind::Other`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectKind {
    /// A human end-user identified by an IAM subject.
    User,
    /// A machine-to-machine client (service account).
    Client,
    /// Internal Mestier code acting on its own behalf (seeds, jobs, CLI,
    /// webhooks). The default [`Authorizer`](crate::Authorizer)
    /// implementation grants every action to system subjects.
    System,
    /// Any subject type Mestier does not know about. Passed through as-is.
    Other(String),
}

impl SubjectKind {
    pub fn as_str(&self) -> &str {
        match self {
            SubjectKind::User => "user",
            SubjectKind::Client => "client",
            SubjectKind::System => "system",
            SubjectKind::Other(s) => s.as_str(),
        }
    }
}

impl From<&str> for SubjectKind {
    fn from(value: &str) -> Self {
        match value {
            "user" => SubjectKind::User,
            "client" => SubjectKind::Client,
            "system" => SubjectKind::System,
            other => SubjectKind::Other(other.to_owned()),
        }
    }
}

/// AuthZen Subject — *who* is acting.
///
/// Wire format ([AuthZen 1.0 §5.1.1]):
///
/// ```json
/// { "type": "user", "id": "alice@acmecorp.com", "properties": { ... } }
/// ```
///
/// `properties` is optional on the wire; an empty map is elided when
/// serializing.
///
/// [AuthZen 1.0 §5.1.1]: https://openid.net/specs/authorization-api-1_0.html
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Properties::is_empty")]
    pub properties: Properties,
}

impl Subject {
    pub fn new(kind: SubjectKind, id: impl Into<String>) -> Self {
        Self {
            r#type: kind.as_str().to_owned(),
            id: id.into(),
            properties: Properties::new(),
        }
    }

    /// Internal Mestier actor (jobs, seeds, CLI, webhooks). The default PDP
    /// allows every action when the subject is of type `system`.
    pub fn system() -> Self {
        Self::new(SubjectKind::System, "mestier")
    }

    pub fn kind(&self) -> SubjectKind {
        SubjectKind::from(self.r#type.as_str())
    }

    pub fn with_property(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    pub fn is_system(&self) -> bool {
        matches!(self.kind(), SubjectKind::System)
    }
}
