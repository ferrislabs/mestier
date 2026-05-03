use serde::{Deserialize, Serialize};

use crate::domain::types::{
    action::Action, context::Context, decision::Decision, resource::Resource, subject::Subject,
};

/// AuthZen single Access Evaluation Request.
///
/// Wire format ([AuthZen 1.0 §5.2.1]):
///
/// ```json
/// {
///   "subject":  { "type": "user",  "id": "alice" },
///   "action":   { "name": "can_read" },
///   "resource": { "type": "account", "id": "123" },
///   "context":  { "time": "..." }
/// }
/// ```
///
/// [AuthZen 1.0 §5.2.1]: https://openid.net/specs/authorization-api-1_0.html
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessEvaluationRequest {
    pub subject: Subject,
    pub action: Action,
    pub resource: Resource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
}

impl AccessEvaluationRequest {
    pub fn new(subject: Subject, action: Action, resource: Resource) -> Self {
        Self {
            subject,
            action,
            resource,
            context: None,
        }
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }
}

/// One entry inside an [`AccessEvaluationsRequest`]. Every field is
/// optional; missing fields fall back to the top-level defaults of the
/// enclosing request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoxedEvaluation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<Subject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<Resource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
}

/// AuthZen Boxed Access Evaluations Request — batch evaluation.
///
/// Wire format ([AuthZen 1.0 §5.3.1]):
///
/// ```json
/// {
///   "subject":     { "type": "user", "id": "alice" },
///   "action":      { "name": "can_read" },
///   "evaluations": [
///     { "resource": { "type": "account", "id": "123" } },
///     { "resource": { "type": "account", "id": "456" } }
///   ]
/// }
/// ```
///
/// Top-level `subject` / `action` / `resource` / `context` act as defaults
/// and are overridden per-entry by the [`BoxedEvaluation`] fields.
///
/// [AuthZen 1.0 §5.3.1]: https://openid.net/specs/authorization-api-1_0.html
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessEvaluationsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<Subject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<Resource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evaluations: Vec<BoxedEvaluation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<crate::domain::types::Properties>,
}

/// Decision returned for one entry of a boxed evaluation. Same shape as
/// [`Decision`]; aliased for symmetry with the request types.
pub type BoxedEvaluationDecision = Decision;

/// AuthZen Boxed Access Evaluations Response.
///
/// Wire format ([AuthZen 1.0 §5.3.2]):
///
/// ```json
/// { "evaluations": [ { "decision": true }, { "decision": false } ] }
/// ```
///
/// [AuthZen 1.0 §5.3.2]: https://openid.net/specs/authorization-api-1_0.html
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessEvaluationsResponse {
    pub evaluations: Vec<BoxedEvaluationDecision>,
}
