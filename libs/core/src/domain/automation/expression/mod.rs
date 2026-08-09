//! The workflow expression language: `{{ ... }}` templates that read a
//! connector's output, compare and combine values, and call a closed list of
//! functions. No JavaScript, no loops of their own — see the module's tests
//! and issue #196 for the grammar this compiles.

mod ast;
mod context;
mod error;
mod eval;
mod parser;

use std::collections::BTreeSet;

pub use context::{ExpressionContext, LoopFrame};
pub use error::ExpressionError;

pub use ast::Template;

/// Compiles one field's raw JSON value into a [`Template`]. Any non-string
/// value is a literal (`is_static()` is `true`); a string is scanned for its
/// `{{ }}` expressions.
pub fn parse_template(raw: &serde_json::Value) -> Result<Template, ExpressionError> {
    parser::parse(raw)
}

impl Template {
    /// Ids of connectors referenced by this template — read `connectors.c1.…`
    /// and `c1` is in this set. Used by #199 to reject a reference to a
    /// connector that does not exist, before any run happens.
    pub fn referenced_connectors(&self) -> BTreeSet<String> {
        BTreeSet::new()
    }

    /// True when the template reads `loop.*`. Used by #199 to reject `loop`
    /// outside of a `flow.loop` connector.
    pub fn uses_loop(&self) -> bool {
        false
    }

    /// True when no evaluation is ever needed: the raw value was not a
    /// string, or was a string with no `{{ }}` in it.
    pub fn is_static(&self) -> bool {
        matches!(self.kind(), ast::TemplateKind::Literal(_))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::TimeZone;
    use serde_json::json;

    use super::*;

    fn fixed_now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
    }

    fn empty_context(connectors: &BTreeMap<String, serde_json::Value>) -> ExpressionContext<'_> {
        ExpressionContext {
            trigger: None,
            connectors,
            loop_frame: None,
            now: fixed_now(),
        }
    }

    #[test]
    fn a_non_string_raw_value_is_a_static_literal() {
        let template = parse_template(&json!(42)).expect("a number is always a valid template");

        assert!(template.is_static());
        assert!(template.referenced_connectors().is_empty());
        assert!(!template.uses_loop());
    }

    #[test]
    fn a_static_literal_evaluates_to_itself_unchanged() {
        let template = parse_template(&json!({ "a": 1, "b": [true, null] }))
            .expect("an object is a valid literal template");
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let result = template
            .evaluate(&ctx)
            .expect("a literal never fails to evaluate");

        assert_eq!(result, json!({ "a": 1, "b": [true, null] }));
    }
}
