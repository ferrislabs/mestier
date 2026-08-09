use serde_json::Value;

use super::ast::Template;
use super::error::ExpressionError;

/// Entry point used by `parse_template`. A non-string value is always a
/// static literal; a string is scanned for `{{ }}` expressions.
pub(crate) fn parse(raw: &Value) -> Result<Template, ExpressionError> {
    match raw {
        Value::String(s) => parse_string_template(s),
        other => Ok(Template::literal(other.clone())),
    }
}

/// A string with no `{{` in it needs no scanning: it is its own literal.
/// The `{{ }}` grammar (whole expressions, interpolation) is built out next.
fn parse_string_template(s: &str) -> Result<Template, ExpressionError> {
    if !s.contains("{{") {
        return Ok(Template::literal(Value::String(s.to_string())));
    }

    unimplemented!("the {{ }} grammar is not parsed yet")
}
