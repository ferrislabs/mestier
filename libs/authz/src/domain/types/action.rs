use serde::{Deserialize, Serialize};

use crate::domain::types::Properties;

/// AuthZen Action — *what* the subject wants to do.
///
/// Wire format ([AuthZen 1.0 §5.1.2]):
///
/// ```json
/// { "name": "organization.update", "properties": { ... } }
/// ```
///
/// Mestier convention is `domain.verb` in lower-case (e.g. `member.remove`,
/// `role.assign`). Names form a public vocabulary — keep them stable.
///
/// [AuthZen 1.0 §5.1.2]: https://openid.net/specs/authorization-api-1_0.html
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    #[serde(default, skip_serializing_if = "Properties::is_empty")]
    pub properties: Properties,
}

impl Action {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            properties: Properties::new(),
        }
    }

    pub fn with_property(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}
