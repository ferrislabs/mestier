use serde::{Deserialize, Serialize};

use crate::domain::types::context::Context;

/// AuthZen Access Evaluation Response.
///
/// Wire format ([AuthZen 1.0 §5.2.2]):
///
/// ```json
/// { "decision": false, "context": { "reason_admin": { "en": "..." } } }
/// ```
///
/// `context` is optional: PDPs use it to carry reasons, obligations, or
/// hints (e.g. localized error messages).
///
/// [AuthZen 1.0 §5.2.2]: https://openid.net/specs/authorization-api-1_0.html
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub decision: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
}

impl Decision {
    pub fn allow() -> Self {
        Self {
            decision: true,
            context: None,
        }
    }

    pub fn deny() -> Self {
        Self {
            decision: false,
            context: None,
        }
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn is_allowed(&self) -> bool {
        self.decision
    }
}
