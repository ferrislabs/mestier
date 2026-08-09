use serde_json::Value;

use super::ast::Template;
use super::error::ExpressionError;

/// Entry point used by `parse_template`. A non-string value is always a
/// static literal; string parsing (the `{{ }}` grammar) is built out next.
pub(crate) fn parse(raw: &Value) -> Result<Template, ExpressionError> {
    match raw {
        Value::String(_) => unimplemented!("string templates are not parsed yet"),
        other => Ok(Template::literal(other.clone())),
    }
}
