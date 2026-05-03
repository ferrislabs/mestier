pub mod action;
pub mod context;
pub mod decision;
pub mod evaluation;
pub mod resource;
pub mod subject;

/// Convenient alias for the JSON object shape used by AuthZen `properties`
/// and `context` fields.
pub type Properties = serde_json::Map<String, serde_json::Value>;
