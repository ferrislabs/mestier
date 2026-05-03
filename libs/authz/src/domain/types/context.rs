use serde::{Deserialize, Serialize};

use crate::domain::types::Properties;

/// AuthZen Context — free-form environmental information attached to an
/// evaluation request or a decision response.
///
/// Wire format ([AuthZen 1.0 §5.1.4]):
///
/// ```json
/// { "time": "2024-05-12T10:30:00Z", "ip": "203.0.113.10" }
/// ```
///
/// Stored as an opaque JSON object; consumers downcast keys they care
/// about.
///
/// [AuthZen 1.0 §5.1.4]: https://openid.net/specs/authorization-api-1_0.html
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Context(pub Properties);

impl Context {
    pub fn new() -> Self {
        Self(Properties::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }
}

impl From<Properties> for Context {
    fn from(value: Properties) -> Self {
        Self(value)
    }
}
