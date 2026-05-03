use serde::{Deserialize, Serialize};

use crate::domain::types::Properties;

/// AuthZen Resource — *what* is being acted upon.
///
/// Wire format ([AuthZen 1.0 §5.1.3]):
///
/// ```json
/// { "type": "organization", "id": "01HZ...", "properties": { ... } }
/// ```
///
/// Mestier convention: `type` is the singular domain noun (`organization`,
/// `member`, `role`, `invoice`, ...) and `id` is the stable database
/// identifier as a string.
///
/// [AuthZen 1.0 §5.1.3]: https://openid.net/specs/authorization-api-1_0.html
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Properties::is_empty")]
    pub properties: Properties,
}

impl Resource {
    pub fn new(r#type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            r#type: r#type.into(),
            id: id.into(),
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
