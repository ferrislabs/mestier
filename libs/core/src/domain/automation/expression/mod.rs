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
        let mut ids = BTreeSet::new();
        for root in self.expr_roots() {
            ast::walk_paths(root, &mut |path| {
                if path.root == ast::PathRoot::Connectors {
                    if let Some(ast::PathSegment::Field(id)) = path.segments.first() {
                        ids.insert(id.clone());
                    }
                }
            });
        }
        ids
    }

    /// True when the template reads `loop.*`. Used by #199 to reject `loop`
    /// outside of a `flow.loop` connector.
    pub fn uses_loop(&self) -> bool {
        let mut found = false;
        for root in self.expr_roots() {
            ast::walk_paths(root, &mut |path| {
                if path.root == ast::PathRoot::Loop {
                    found = true;
                }
            });
        }
        found
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
    use serde_json::{Value, json};

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

    #[test]
    fn a_plain_string_with_no_braces_is_a_static_literal() {
        let template =
            parse_template(&json!("hello world")).expect("plain text is a valid template");
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        assert!(template.is_static());
        assert_eq!(
            template.evaluate(&ctx).expect("a literal never fails"),
            json!("hello world")
        );
    }

    #[test]
    fn a_whole_expression_is_not_static() {
        let template = parse_template(&json!("{{ 42 }}")).expect("valid expression");

        assert!(
            !template.is_static(),
            "a whole expression always needs evaluation"
        );
    }

    #[test]
    fn whole_expression_literals_preserve_their_json_type() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let cases: &[(&str, Value)] = &[
            ("{{ 42 }}", json!(42)),
            ("{{ 3.5 }}", json!(3.5)),
            ("{{ -7 }}", json!(-7)),
            ("{{ \"abc\" }}", json!("abc")),
            ("{{ 'abc' }}", json!("abc")),
            ("{{ true }}", json!(true)),
            ("{{ false }}", json!(false)),
            ("{{ null }}", Value::Null),
        ];

        for (raw, expected) in cases {
            let template = parse_template(&json!(raw))
                .unwrap_or_else(|e| panic!("parsing `{raw}` failed: {e}"));
            let result = template
                .evaluate(&ctx)
                .unwrap_or_else(|e| panic!("evaluating `{raw}` failed: {e}"));
            assert_eq!(&result, expected, "for `{raw}`");
        }
    }

    #[test]
    fn surrounding_whitespace_does_not_prevent_a_whole_expression() {
        let template = parse_template(&json!("   {{ 42 }}   ")).expect("valid expression");
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        assert!(!template.is_static());
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(42));
    }

    #[test]
    fn an_unterminated_brace_is_a_syntax_error_naming_its_position() {
        let err = parse_template(&json!("hello {{ 42 ")).expect_err("must fail to parse");

        assert_eq!(
            err,
            ExpressionError::Syntax {
                position: 6,
                message: "unterminated `{{`".to_string(),
            }
        );
    }

    #[test]
    fn a_trigger_path_reads_a_nested_field() {
        let trigger = json!({ "payload": { "quote": { "total": 42 } } });
        let connectors = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: Some(&trigger),
            connectors: &connectors,
            loop_frame: None,
            now: fixed_now(),
        };
        let template = parse_template(&json!("{{ trigger.payload.quote.total }}"))
            .expect("valid path expression");

        assert_eq!(template.evaluate(&ctx).expect("resolves"), json!(42));
    }

    #[test]
    fn a_connectors_path_reads_a_nested_array_element() {
        let mut connectors = BTreeMap::new();
        connectors.insert(
            "c1".to_string(),
            json!({ "output": { "lines": [{ "total": 12 }, { "total": 34 }] } }),
        );
        let ctx = empty_context(&connectors);
        let template = parse_template(&json!("{{ connectors.c1.output.lines[1].total }}"))
            .expect("valid path expression");

        assert_eq!(template.evaluate(&ctx).expect("resolves"), json!(34));
    }

    #[test]
    fn a_bare_trigger_path_returns_the_whole_payload() {
        let trigger = json!({ "quote_id": "q-1" });
        let connectors = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: Some(&trigger),
            connectors: &connectors,
            loop_frame: None,
            now: fixed_now(),
        };
        let template = parse_template(&json!("{{ trigger }}")).expect("valid path expression");

        assert_eq!(
            template.evaluate(&ctx).expect("resolves"),
            json!({ "quote_id": "q-1" })
        );
    }

    #[test]
    fn a_missing_field_fails_naming_the_full_path() {
        let mut connectors = BTreeMap::new();
        connectors.insert("c1".to_string(), json!({ "output": { "total": 1 } }));
        let ctx = empty_context(&connectors);
        let template = parse_template(&json!("{{ connectors.c1.output.items }}"))
            .expect("valid path expression");

        let err = template
            .evaluate(&ctx)
            .expect_err("the field does not exist");

        assert_eq!(
            err,
            ExpressionError::MissingPath {
                path: "connectors.c1.output.items".to_string(),
            }
        );
    }

    #[test]
    fn an_absent_trigger_fails_naming_the_path() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        let template =
            parse_template(&json!("{{ trigger.quote_id }}")).expect("valid path expression");

        let err = template.evaluate(&ctx).expect_err("no trigger in context");

        assert_eq!(
            err,
            ExpressionError::MissingPath {
                path: "trigger.quote_id".to_string(),
            }
        );
    }

    #[test]
    fn an_unknown_connector_id_fails_naming_the_path() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        let template =
            parse_template(&json!("{{ connectors.missing.output }}")).expect("valid path");

        let err = template.evaluate(&ctx).expect_err("no such connector");

        assert_eq!(
            err,
            ExpressionError::MissingPath {
                path: "connectors.missing.output".to_string(),
            }
        );
    }

    #[test]
    fn connectors_alone_without_an_id_is_a_syntax_error() {
        let err = parse_template(&json!("{{ connectors }}")).expect_err("no connector id");

        assert!(matches!(err, ExpressionError::Syntax { .. }), "{err:?}");
    }

    #[test]
    fn referenced_connectors_names_the_connector_id_of_a_path() {
        let template = parse_template(&json!("{{ connectors.c1.output.a }}")).expect("valid path");

        let ids: Vec<_> = template.referenced_connectors().into_iter().collect();
        assert_eq!(ids, vec!["c1".to_string()]);
    }

    #[test]
    fn a_trigger_or_loop_path_is_never_a_referenced_connector() {
        let template = parse_template(&json!("{{ trigger.quote_id }}")).expect("valid path");

        assert!(template.referenced_connectors().is_empty());
    }

    #[test]
    fn loop_item_and_index_resolve_inside_a_loop_frame() {
        let item = json!({ "name": "brioche" });
        let connectors = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: None,
            connectors: &connectors,
            loop_frame: Some(LoopFrame {
                item: &item,
                index: 3,
            }),
            now: fixed_now(),
        };

        let name_template =
            parse_template(&json!("{{ loop.item.name }}")).expect("valid loop path");
        let index_template = parse_template(&json!("{{ loop.index }}")).expect("valid loop path");

        assert!(name_template.uses_loop());
        assert_eq!(
            name_template.evaluate(&ctx).expect("resolves"),
            json!("brioche")
        );
        assert_eq!(index_template.evaluate(&ctx).expect("resolves"), json!(3));
    }

    #[test]
    fn loop_outside_a_loop_frame_fails_explicitly() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        let template = parse_template(&json!("{{ loop.item }}")).expect("valid loop path");

        let err = template
            .evaluate(&ctx)
            .expect_err("no loop frame in context");

        assert_eq!(err, ExpressionError::LoopOutsideLoop);
    }

    #[test]
    fn comparison_operators_evaluate_on_numbers() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let cases = [
            ("{{ 1 == 1 }}", true),
            ("{{ 1 == 2 }}", false),
            ("{{ 1 != 2 }}", true),
            ("{{ 1 < 2 }}", true),
            ("{{ 2 < 1 }}", false),
            ("{{ 1 <= 1 }}", true),
            ("{{ 2 <= 1 }}", false),
            ("{{ 2 > 1 }}", true),
            ("{{ 2 >= 2 }}", true),
        ];

        for (raw, expected) in cases {
            let template =
                parse_template(&json!(raw)).unwrap_or_else(|e| panic!("{raw} failed: {e}"));
            let result = template
                .evaluate(&ctx)
                .unwrap_or_else(|e| panic!("{raw} failed: {e}"));
            assert_eq!(result, json!(expected), "for {raw}");
        }
    }

    #[test]
    fn equality_also_works_on_strings_and_null() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let template = parse_template(&json!("{{ \"a\" == \"a\" }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(true));

        let template = parse_template(&json!("{{ null == null }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(true));
    }

    #[test]
    fn boolean_operators_and_not_evaluate() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let cases = [
            ("{{ not true }}", false),
            ("{{ not false }}", true),
            ("{{ true and false }}", false),
            ("{{ true and true }}", true),
            ("{{ true or false }}", true),
            ("{{ false or false }}", false),
        ];

        for (raw, expected) in cases {
            let template =
                parse_template(&json!(raw)).unwrap_or_else(|e| panic!("{raw} failed: {e}"));
            let result = template
                .evaluate(&ctx)
                .unwrap_or_else(|e| panic!("{raw} failed: {e}"));
            assert_eq!(result, json!(expected), "for {raw}");
        }
    }

    #[test]
    fn not_binds_tighter_than_comparisons_which_bind_tighter_than_and_which_binds_tighter_than_or()
    {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        // (not false) and false == false, not `not (false and false)` == true.
        let template = parse_template(&json!("{{ not false and false }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(false));

        // (1 == 1) or false, not `1 == (1 or false)` which would be a syntax error.
        let template = parse_template(&json!("{{ 1 == 1 or false }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(true));
    }

    #[test]
    fn parentheses_override_precedence() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let template = parse_template(&json!("{{ not (1 == 1) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(false));

        let template = parse_template(&json!("{{ (1 == 1) and (2 == 3) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(false));
    }

    #[test]
    fn and_short_circuits_without_evaluating_the_right_operand() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        // connectors.missing does not exist: if the right side were
        // evaluated, this would fail with MissingPath instead.
        let template =
            parse_template(&json!("{{ false and connectors.missing.output }}")).expect("valid");

        assert_eq!(
            template.evaluate(&ctx).expect("short-circuits"),
            json!(false)
        );
    }

    #[test]
    fn or_short_circuits_without_evaluating_the_right_operand() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        let template =
            parse_template(&json!("{{ true or connectors.missing.output }}")).expect("valid");

        assert_eq!(
            template.evaluate(&ctx).expect("short-circuits"),
            json!(true)
        );
    }

    #[test]
    fn comparing_a_number_to_a_string_is_a_type_mismatch() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        let template = parse_template(&json!("{{ 1 < \"a\" }}")).expect("valid syntax");

        let err = template.evaluate(&ctx).expect_err("not comparable");

        assert_eq!(
            err,
            ExpressionError::TypeMismatch {
                path: "\"a\"".to_string(),
                expected: "number",
                got: "string",
            }
        );
    }

    #[test]
    fn and_on_a_non_boolean_is_a_type_mismatch() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        let template = parse_template(&json!("{{ 1 and true }}")).expect("valid syntax");

        let err = template.evaluate(&ctx).expect_err("1 is not a boolean");

        assert_eq!(
            err,
            ExpressionError::TypeMismatch {
                path: "1".to_string(),
                expected: "boolean",
                got: "number",
            }
        );
    }

    #[test]
    fn an_unknown_function_name_is_rejected_at_parse_time() {
        let err = parse_template(&json!("{{ frobnicate(1) }}")).expect_err("no such function");

        assert_eq!(
            err,
            ExpressionError::UnknownFunction {
                name: "frobnicate".to_string(),
            }
        );
    }

    #[test]
    fn a_wrong_argument_count_is_rejected_at_parse_time() {
        let err = parse_template(&json!("{{ upper() }}")).expect_err("upper takes 1 argument");
        assert_eq!(
            err,
            ExpressionError::Arity {
                function: "upper".to_string(),
                expected: 1,
                got: 0,
            }
        );

        let err = parse_template(&json!("{{ upper(\"a\", \"b\") }}"))
            .expect_err("upper takes 1 argument");
        assert_eq!(
            err,
            ExpressionError::Arity {
                function: "upper".to_string(),
                expected: 1,
                got: 2,
            }
        );
    }

    #[test]
    fn string_functions_evaluate() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let cases = [
            ("{{ upper(\"abc\") }}", json!("ABC")),
            ("{{ lower(\"ABC\") }}", json!("abc")),
            ("{{ trim(\"  hi  \") }}", json!("hi")),
            ("{{ starts_with(\"hello\", \"he\") }}", json!(true)),
            ("{{ starts_with(\"hello\", \"lo\") }}", json!(false)),
            ("{{ contains(\"hello world\", \"wor\") }}", json!(true)),
            ("{{ contains(\"hello world\", \"xyz\") }}", json!(false)),
        ];

        for (raw, expected) in cases {
            let template =
                parse_template(&json!(raw)).unwrap_or_else(|e| panic!("{raw} failed: {e}"));
            let result = template
                .evaluate(&ctx)
                .unwrap_or_else(|e| panic!("{raw} failed: {e}"));
            assert_eq!(result, expected, "for {raw}");
        }
    }

    #[test]
    fn len_counts_strings_arrays_and_objects() {
        let mut connectors = BTreeMap::new();
        connectors.insert(
            "c1".to_string(),
            json!({ "output": { "items": [1, 2, 3] } }),
        );
        let ctx = empty_context(&connectors);

        let template = parse_template(&json!("{{ len(\"abc\") }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(3));

        let template =
            parse_template(&json!("{{ len(connectors.c1.output.items) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(3));
    }

    #[test]
    fn contains_also_works_on_arrays() {
        let mut connectors = BTreeMap::new();
        connectors.insert(
            "c1".to_string(),
            json!({ "output": { "items": [1, 2, 3] } }),
        );
        let ctx = empty_context(&connectors);

        let template =
            parse_template(&json!("{{ contains(connectors.c1.output.items, 2) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(true));

        let template =
            parse_template(&json!("{{ contains(connectors.c1.output.items, 5) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(false));
    }

    #[test]
    fn to_number_and_to_string_convert() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let template = parse_template(&json!("{{ to_number(\"42\") }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(42));

        let template = parse_template(&json!("{{ to_number(\"3.5\") }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(3.5));

        let template = parse_template(&json!("{{ to_string(42) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!("42"));
    }

    #[test]
    fn to_number_on_a_non_numeric_string_is_a_type_mismatch() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        let template = parse_template(&json!("{{ to_number(\"abc\") }}")).expect("valid syntax");

        let err = template.evaluate(&ctx).expect_err("not numeric");
        assert!(
            matches!(err, ExpressionError::TypeMismatch { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn round_rounds_to_the_given_number_of_decimals() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let template = parse_template(&json!("{{ round(3.14159, 2) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(3.14));

        let template = parse_template(&json!("{{ round(2.5, 0) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(3));
    }

    #[test]
    fn concat_stringifies_and_joins_every_argument() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        let template = parse_template(&json!("{{ concat(\"total: \", 42, \" (\", true, \")\") }}"))
            .expect("valid");

        assert_eq!(
            template.evaluate(&ctx).expect("evaluates"),
            json!("total: 42 (true)")
        );
    }

    #[test]
    fn json_serializes_arrays_and_objects_as_a_string() {
        let mut connectors = BTreeMap::new();
        connectors.insert("c1".to_string(), json!({ "output": { "a": 1 } }));
        let ctx = empty_context(&connectors);
        let template = parse_template(&json!("{{ json(connectors.c1.output) }}")).expect("valid");

        assert_eq!(
            template.evaluate(&ctx).expect("evaluates"),
            json!("{\"a\":1}")
        );
    }

    #[test]
    fn now_reads_the_injected_clock_not_the_system_clock() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);
        let template =
            parse_template(&json!("{{ format_date(now(), \"%Y-%m-%d\") }}")).expect("valid");

        assert_eq!(
            template.evaluate(&ctx).expect("evaluates"),
            json!("2026-01-01")
        );
    }

    #[test]
    fn format_date_formats_a_path_value_too() {
        let trigger = json!({ "created_at": "2025-06-15T10:30:00Z" });
        let connectors = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: Some(&trigger),
            connectors: &connectors,
            loop_frame: None,
            now: fixed_now(),
        };
        let template = parse_template(&json!(
            "{{ format_date(trigger.created_at, \"%Y/%m/%d\") }}"
        ))
        .expect("valid");

        assert_eq!(
            template.evaluate(&ctx).expect("evaluates"),
            json!("2025/06/15")
        );
    }

    #[test]
    fn default_returns_the_fallback_only_when_the_path_is_missing() {
        let mut connectors = BTreeMap::new();
        connectors.insert("c1".to_string(), json!({ "output": { "name": "brioche" } }));
        let ctx = empty_context(&connectors);

        let template = parse_template(&json!(
            "{{ default(connectors.c1.output.missing, \"fallback\") }}"
        ))
        .expect("valid");
        assert_eq!(
            template.evaluate(&ctx).expect("evaluates"),
            json!("fallback")
        );

        let template = parse_template(&json!(
            "{{ default(connectors.c1.output.name, \"fallback\") }}"
        ))
        .expect("valid");
        assert_eq!(
            template.evaluate(&ctx).expect("evaluates"),
            json!("brioche")
        );
    }

    #[test]
    fn default_does_not_catch_a_type_mismatch_or_an_unknown_function() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let template =
            parse_template(&json!("{{ default(1 and true, \"fallback\") }}")).expect("valid");
        let err = template.evaluate(&ctx).expect_err("must not be caught");
        assert!(
            matches!(err, ExpressionError::TypeMismatch { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn an_array_alone_stays_an_array_but_interpolated_it_becomes_a_string() {
        let mut connectors = BTreeMap::new();
        connectors.insert(
            "c1".to_string(),
            json!({ "output": { "items": [1, 2, 3] } }),
        );
        let ctx = empty_context(&connectors);

        let whole = parse_template(&json!("{{ connectors.c1.output.items }}")).expect("valid");
        assert_eq!(
            whole.evaluate(&ctx).expect("evaluates"),
            json!([1, 2, 3]),
            "a whole expression keeps its JSON type"
        );

        let interpolated =
            parse_template(&json!("Items: {{ connectors.c1.output.items }}")).expect("valid");
        assert_eq!(
            interpolated.evaluate(&ctx).expect("evaluates"),
            json!("Items: [1,2,3]"),
            "the same expression in a sentence always renders a string"
        );
    }

    #[test]
    fn interpolation_stringifies_every_json_type_without_requoting_strings() {
        let connectors = BTreeMap::new();
        let ctx = empty_context(&connectors);

        let cases = [
            ("Hello {{ \"World\" }}!", "Hello World!"),
            ("Flag: {{ true }}", "Flag: true"),
            ("Nothing: {{ null }}", "Nothing: null"),
            ("Count: {{ 3 }}", "Count: 3"),
        ];
        for (raw, expected) in cases {
            let template =
                parse_template(&json!(raw)).unwrap_or_else(|e| panic!("{raw} failed: {e}"));
            let result = template
                .evaluate(&ctx)
                .unwrap_or_else(|e| panic!("{raw} failed: {e}"));
            assert_eq!(result, json!(expected), "for {raw}");
        }
    }

    #[test]
    fn interpolation_is_not_static_and_referenced_connectors_covers_every_expression() {
        let mut connectors = BTreeMap::new();
        connectors.insert("c1".to_string(), json!({ "output": { "a": 1 } }));
        connectors.insert("c2".to_string(), json!({ "output": { "b": 2 } }));
        let ctx = empty_context(&connectors);

        let template = parse_template(&json!(
            "{{ connectors.c1.output.a }} and {{ connectors.c2.output.b }} and {{ connectors.c1.output.a }}"
        ))
        .expect("valid");

        assert!(!template.is_static());
        assert_eq!(
            template.evaluate(&ctx).expect("evaluates"),
            json!("1 and 2 and 1")
        );
        let ids: Vec<_> = template.referenced_connectors().into_iter().collect();
        assert_eq!(ids, vec!["c1".to_string(), "c2".to_string()]);
    }

    #[test]
    fn uses_loop_is_true_when_loop_is_read_inside_an_interpolated_string() {
        let template =
            parse_template(&json!("Item #{{ loop.index }}: {{ loop.item.name }}")).expect("valid");

        assert!(template.uses_loop());
    }
}
