//! The workflow expression language: `{{ ... }}` templates that read a
//! node's items by name, compare and combine values, and call a closed list of
//! functions. No JavaScript, no loops of their own — see the module's tests
//! and issue #196 for the grammar this compiles.
//!
//! `domain` is `pub(crate)` (see `libs/core/src/lib.rs`), so this module's
//! surface reaches the outside only through the re-export there. That
//! re-export is what #199 and #200 will consume, and it is also what keeps
//! rustc from reading the whole language as dead code while it has no caller
//! yet — a blanket `allow(dead_code)` would have hidden the real thing it is
//! there to catch.

mod ast;
mod budget;
mod context;
mod error;
mod eval;
mod parser;

use std::collections::BTreeSet;

pub use budget::EvalBudget;
pub(crate) use budget::json_size;
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
    pub fn referenced_nodes(&self) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for root in self.expr_roots() {
            ast::walk_paths(root, &mut |path| {
                if let ast::PathRoot::Node { name, .. } = &path.root {
                    names.insert(name.clone());
                }
            });
        }
        names
    }

    /// True when the template reads `loop.*`. Used by #199 to reject `loop`
    /// outside of a `flow.loop` connector.
    pub fn uses_loop(&self) -> bool {
        let mut found = false;
        for root in self.expr_roots() {
            ast::walk_paths(root, &mut |path| {
                if matches!(path.root, ast::PathRoot::Loop) {
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

    use super::budget::MAX_RESOLVED_BYTES;
    use super::*;

    fn fixed_now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
    }

    fn empty_context(nodes: &BTreeMap<String, Vec<Value>>) -> ExpressionContext<'_> {
        ExpressionContext {
            trigger: None,
            nodes,
            position: 0,
            loop_frame: None,
            now: fixed_now(),
        }
    }

    fn nodes_of(entries: &[(&str, Vec<Value>)]) -> BTreeMap<String, Vec<Value>> {
        entries
            .iter()
            .map(|(name, items)| ((*name).to_string(), items.clone()))
            .collect()
    }

    fn at(nodes: &BTreeMap<String, Vec<Value>>, position: usize) -> ExpressionContext<'_> {
        ExpressionContext {
            position,
            ..empty_context(nodes)
        }
    }

    fn read(ctx: &ExpressionContext<'_>, source: &str) -> Result<Value, ExpressionError> {
        parse_template(&json!(source))
            .unwrap_or_else(|error| panic!("`{source}` must parse: {error}"))
            .evaluate(ctx)
    }

    #[test]
    fn a_non_string_raw_value_is_a_static_literal() {
        let template = parse_template(&json!(42)).expect("a number is always a valid template");

        assert!(template.is_static());
        assert!(template.referenced_nodes().is_empty());
        assert!(!template.uses_loop());
    }

    #[test]
    fn a_static_literal_evaluates_to_itself_unchanged() {
        let template = parse_template(&json!({ "a": 1, "b": [true, null] }))
            .expect("an object is a valid literal template");
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

        let result = template
            .evaluate(&ctx)
            .expect("a literal never fails to evaluate");

        assert_eq!(result, json!({ "a": 1, "b": [true, null] }));
    }

    #[test]
    fn a_plain_string_with_no_braces_is_a_static_literal() {
        let template =
            parse_template(&json!("hello world")).expect("plain text is a valid template");
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

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
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

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
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

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
        let nodes = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: Some(&trigger),
            nodes: &nodes,
            position: 0,
            loop_frame: None,
            now: fixed_now(),
        };
        let template = parse_template(&json!("{{ trigger.payload.quote.total }}"))
            .expect("valid path expression");

        assert_eq!(template.evaluate(&ctx).expect("resolves"), json!(42));
    }

    #[test]
    fn a_node_path_reads_a_nested_array_element() {
        let nodes = nodes_of(&[(
            "Rechercher",
            vec![json!({ "lines": [{ "total": 12 }, { "total": 34 }] })],
        )]);
        let ctx = empty_context(&nodes);

        assert_eq!(
            read(&ctx, "{{ $('Rechercher').item.json.lines[1].total }}").unwrap(),
            json!(34)
        );
    }

    #[test]
    fn a_bare_trigger_path_returns_the_whole_payload() {
        let trigger = json!({ "quote_id": "q-1" });
        let nodes = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: Some(&trigger),
            nodes: &nodes,
            position: 0,
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
        let nodes = nodes_of(&[("Rechercher", vec![json!({ "total": 1 })])]);
        let ctx = empty_context(&nodes);

        let err = read(&ctx, "{{ $('Rechercher').item.json.items }}").unwrap_err();

        assert_eq!(
            err,
            ExpressionError::MissingPath {
                path: "$('Rechercher').item.json.items".to_string(),
            }
        );
    }

    #[test]
    fn an_absent_trigger_fails_naming_the_path() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
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
    fn an_unknown_node_fails_naming_it() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

        let err = read(&ctx, "{{ $('Absent').item.json }}").unwrap_err();

        assert_eq!(
            err,
            ExpressionError::UnknownNode {
                name: "Absent".to_string(),
            }
        );
    }

    #[test]
    fn the_connectors_root_is_refused_with_the_new_syntax() {
        for source in [
            "{{ connectors.c1.output.total }}",
            "{{ connectors }}",
            "Total {{ upper(connectors.c1.output.name) }}",
        ] {
            let err = syntax_error(source);

            assert!(
                matches!(err, ExpressionError::LegacyConnectorsRoot { .. }),
                "{source}: {err:?}"
            );
            assert!(
                err.to_string().contains("$('Nom du nœud').item.json"),
                "{err}"
            );
        }
    }

    #[test]
    fn referenced_nodes_names_the_node_of_a_path() {
        let template =
            parse_template(&json!("{{ $('Rechercher').item.json.a }}")).expect("valid path");

        let names: Vec<_> = template.referenced_nodes().into_iter().collect();
        assert_eq!(names, vec!["Rechercher".to_string()]);
    }

    #[test]
    fn a_trigger_or_loop_path_is_never_a_referenced_node() {
        let template = parse_template(&json!("{{ trigger.quote_id }}")).expect("valid path");
        let looped = parse_template(&json!("{{ loop.item.id }}")).expect("valid path");

        assert!(template.referenced_nodes().is_empty());
        assert!(looped.referenced_nodes().is_empty());
    }

    #[test]
    fn loop_item_and_index_resolve_inside_a_loop_frame() {
        let item = json!({ "name": "brioche" });
        let nodes = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: None,
            nodes: &nodes,
            position: 0,
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
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
        let template = parse_template(&json!("{{ loop.item }}")).expect("valid loop path");

        let err = template
            .evaluate(&ctx)
            .expect_err("no loop frame in context");

        assert_eq!(err, ExpressionError::LoopOutsideLoop);
    }

    #[test]
    fn comparison_operators_evaluate_on_numbers() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

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
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

        let template = parse_template(&json!("{{ \"a\" == \"a\" }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(true));

        let template = parse_template(&json!("{{ null == null }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(true));
    }

    #[test]
    fn boolean_operators_and_not_evaluate() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

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
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

        // (not false) and false == false, not `not (false and false)` == true.
        let template = parse_template(&json!("{{ not false and false }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(false));

        // (1 == 1) or false, not `1 == (1 or false)` which would be a syntax error.
        let template = parse_template(&json!("{{ 1 == 1 or false }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(true));
    }

    #[test]
    fn parentheses_override_precedence() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

        let template = parse_template(&json!("{{ not (1 == 1) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(false));

        let template = parse_template(&json!("{{ (1 == 1) and (2 == 3) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(false));
    }

    #[test]
    fn and_short_circuits_without_evaluating_the_right_operand() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
        let template =
            parse_template(&json!("{{ false and $('Missing').item.json.ok }}")).expect("valid");

        assert_eq!(
            template.evaluate(&ctx).expect("short-circuits"),
            json!(false)
        );
    }

    #[test]
    fn or_short_circuits_without_evaluating_the_right_operand() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
        let template =
            parse_template(&json!("{{ true or $('Missing').item.json.ok }}")).expect("valid");

        assert_eq!(
            template.evaluate(&ctx).expect("short-circuits"),
            json!(true)
        );
    }

    #[test]
    fn comparing_a_number_to_a_string_is_a_type_mismatch() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
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
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
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
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

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
        let nodes = nodes_of(&[("Source", vec![json!({ "items": [1, 2, 3] })])]);
        let ctx = empty_context(&nodes);

        assert_eq!(read(&ctx, "{{ len(\"abc\") }}").unwrap(), json!(3));
        assert_eq!(
            read(&ctx, "{{ len($('Source').item.json.items) }}").unwrap(),
            json!(3)
        );
        assert_eq!(
            read(&ctx, "{{ len($('Source').all()) }}").unwrap(),
            json!(1)
        );
    }

    #[test]
    fn contains_also_works_on_arrays() {
        let nodes = nodes_of(&[("Source", vec![json!({ "items": [1, 2, 3] })])]);
        let ctx = empty_context(&nodes);

        assert_eq!(
            read(&ctx, "{{ contains($('Source').item.json.items, 2) }}").unwrap(),
            json!(true)
        );
        assert_eq!(
            read(&ctx, "{{ contains($('Source').item.json.items, 5) }}").unwrap(),
            json!(false)
        );
    }

    #[test]
    fn to_number_and_to_string_convert() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

        let template = parse_template(&json!("{{ to_number(\"42\") }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(42));

        let template = parse_template(&json!("{{ to_number(\"3.5\") }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(3.5));

        let template = parse_template(&json!("{{ to_string(42) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!("42"));
    }

    #[test]
    fn to_number_on_a_non_numeric_string_is_a_type_mismatch() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
        let template = parse_template(&json!("{{ to_number(\"abc\") }}")).expect("valid syntax");

        let err = template.evaluate(&ctx).expect_err("not numeric");
        assert!(
            matches!(err, ExpressionError::TypeMismatch { .. }),
            "{err:?}"
        );
    }

    #[test]
    // 3.14 here is round(3.14159, 2), not an approximation of `PI` — the
    // collision with clippy's known-constant list is coincidental.
    #[allow(clippy::approx_constant)]
    fn round_rounds_to_the_given_number_of_decimals() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

        let template = parse_template(&json!("{{ round(3.14159, 2) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(3.14));

        let template = parse_template(&json!("{{ round(2.5, 0) }}")).expect("valid");
        assert_eq!(template.evaluate(&ctx).expect("evaluates"), json!(3));
    }

    #[test]
    fn concat_stringifies_and_joins_every_argument() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
        let template = parse_template(&json!("{{ concat(\"total: \", 42, \" (\", true, \")\") }}"))
            .expect("valid");

        assert_eq!(
            template.evaluate(&ctx).expect("evaluates"),
            json!("total: 42 (true)")
        );
    }

    #[test]
    fn json_serializes_arrays_and_objects_as_a_string() {
        let nodes = nodes_of(&[("Source", vec![json!({ "a": 1 })])]);
        let ctx = empty_context(&nodes);

        assert_eq!(
            read(&ctx, "{{ json($('Source').item.json) }}").unwrap(),
            json!("{\"a\":1}")
        );
    }

    #[test]
    fn now_reads_the_injected_clock_not_the_system_clock() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
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
        let nodes = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: Some(&trigger),
            nodes: &nodes,
            position: 0,
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
        let nodes = nodes_of(&[("Source", vec![json!({ "name": "brioche" })])]);
        let ctx = empty_context(&nodes);

        assert_eq!(
            read(
                &ctx,
                "{{ default($('Source').item.json.missing, \"fallback\") }}"
            )
            .unwrap(),
            json!("fallback")
        );
        assert_eq!(
            read(
                &ctx,
                "{{ default($('Source').item.json.name, \"fallback\") }}"
            )
            .unwrap(),
            json!("brioche")
        );
    }

    #[test]
    fn default_does_not_catch_a_type_mismatch_or_an_unknown_function() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

        let template =
            parse_template(&json!("{{ default(1 and true, \"fallback\") }}")).expect("valid");
        let err = template.evaluate(&ctx).expect_err("must not be caught");
        assert!(
            matches!(err, ExpressionError::TypeMismatch { .. }),
            "{err:?}"
        );

        // An unknown function cannot reach `default` at all: it is refused
        // while parsing, so no fallback can ever mask a typo in a name.
        let err = parse_template(&json!("{{ default(nope(1), \"fallback\") }}"))
            .expect_err("must not parse");
        assert!(
            matches!(err, ExpressionError::UnknownFunction { ref name } if name == "nope"),
            "{err:?}"
        );
    }

    #[test]
    fn an_array_alone_stays_an_array_but_interpolated_it_becomes_a_string() {
        let nodes = nodes_of(&[("Source", vec![json!({ "items": [1, 2, 3] })])]);
        let ctx = empty_context(&nodes);

        assert_eq!(
            read(&ctx, "{{ $('Source').item.json.items }}").unwrap(),
            json!([1, 2, 3]),
            "a whole expression keeps its JSON type"
        );
        assert_eq!(
            read(&ctx, "Items: {{ $('Source').item.json.items }}").unwrap(),
            json!("Items: [1,2,3]"),
            "the same expression in a sentence always renders a string"
        );
    }

    #[test]
    fn interpolation_stringifies_every_json_type_without_requoting_strings() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);

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
    fn interpolation_is_not_static_and_referenced_nodes_covers_every_expression() {
        let nodes = nodes_of(&[
            ("Premier", vec![json!({ "a": 1 })]),
            ("Second", vec![json!({ "b": 2 })]),
        ]);
        let ctx = empty_context(&nodes);

        let template = parse_template(&json!(
            "{{ $('Premier').item.json.a }} and {{ $('Second').item.json.b }} and {{ $('Premier').item.json.a }}"
        ))
        .expect("valid");

        assert!(!template.is_static());
        assert_eq!(
            template.evaluate(&ctx).expect("evaluates"),
            json!("1 and 2 and 1")
        );
        let names: Vec<_> = template.referenced_nodes().into_iter().collect();
        assert_eq!(names, vec!["Premier".to_string(), "Second".to_string()]);
    }

    #[test]
    fn uses_loop_is_true_when_loop_is_read_inside_an_interpolated_string() {
        let template =
            parse_template(&json!("Item #{{ loop.index }}: {{ loop.item.name }}")).expect("valid");

        assert!(template.uses_loop());
    }

    fn payload_context<'a>(
        trigger: &'a Value,
        nodes: &'a BTreeMap<String, Vec<Value>>,
    ) -> ExpressionContext<'a> {
        ExpressionContext {
            trigger: Some(trigger),
            nodes,
            position: 0,
            loop_frame: None,
            now: fixed_now(),
        }
    }

    fn syntax_error(source: &str) -> ExpressionError {
        parse_template(&json!(source)).expect_err("the template must be refused")
    }

    #[test]
    fn parentheses_nested_past_the_ceiling_are_a_syntax_error_not_a_stack_overflow() {
        let depth = 100_000;
        let source = format!("{{{{ {}1{} }}}}", "(".repeat(depth), ")".repeat(depth));

        let error = syntax_error(&source);

        assert!(matches!(error, ExpressionError::Syntax { .. }), "{error:?}");
    }

    #[test]
    fn a_run_of_nots_calls_and_chained_operators_is_bounded_too() {
        let nots = format!("{{{{ {}true }}}}", "not ".repeat(100_000));
        let calls = format!(
            "{{{{ {}1{} }}}}",
            "upper(".repeat(100_000),
            ")".repeat(100_000)
        );
        let chain = format!("{{{{ true{} }}}}", " or true".repeat(100_000));

        for source in [nots, calls, chain] {
            let error = syntax_error(&source);

            assert!(matches!(error, ExpressionError::Syntax { .. }), "{error:?}");
        }
    }

    #[test]
    fn nesting_within_the_ceiling_still_evaluates() {
        let nodes = BTreeMap::new();
        let ctx = empty_context(&nodes);
        let source = format!("{{{{ {}1{} }}}}", "(".repeat(30), ")".repeat(30));
        let chain = format!("{{{{ false{} }}}}", " or true".repeat(30));

        let nested = parse_template(&json!(source)).expect("shallow nesting parses");
        let chained = parse_template(&json!(chain)).expect("a short chain parses");

        assert_eq!(nested.evaluate(&ctx).unwrap(), json!(1));
        assert_eq!(chained.evaluate(&ctx).unwrap(), json!(true));
    }

    #[test]
    fn a_path_clones_its_leaf_only_never_the_root_it_hangs_from() {
        let trigger = json!({ "big": "x".repeat(2 * 1024 * 1024), "small": 7 });
        let nodes = BTreeMap::new();
        let ctx = payload_context(&trigger, &nodes);
        let small = parse_template(&json!("{{ trigger.small }}")).unwrap();
        let big = parse_template(&json!("{{ trigger.big }}")).unwrap();

        let mut budget = EvalBudget::new(1024);
        let value = small.evaluate_within(&ctx, &mut budget).unwrap();
        let refused = big
            .evaluate_within(&ctx, &mut EvalBudget::new(1024))
            .unwrap_err();

        assert_eq!(value, json!(7));
        assert_eq!(refused, ExpressionError::TooLarge { limit: 1024 });
    }

    #[test]
    fn eighty_thousand_references_to_a_megabyte_leaf_are_refused_long_before_they_are_built() {
        let trigger = json!({ "p": "x".repeat(1024 * 1024) });
        let nodes = BTreeMap::new();
        let ctx = payload_context(&trigger, &nodes);
        let source = "{{trigger.p}}".repeat(80_000);
        let template = parse_template(&json!(source)).unwrap();
        let started = std::time::Instant::now();

        let error = template.evaluate(&ctx).unwrap_err();

        assert_eq!(
            error,
            ExpressionError::TooLarge {
                limit: MAX_RESOLVED_BYTES
            }
        );
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }

    #[test]
    fn every_function_that_builds_a_string_counts_what_it_builds() {
        let trigger = json!({ "p": "x".repeat(600) });
        let nodes = BTreeMap::new();
        let ctx = payload_context(&trigger, &nodes);

        for source in [
            "{{ concat(trigger.p, trigger.p) }}",
            "{{ to_string(trigger.p) }}",
            "{{ upper(trigger.p) }}",
            "{{ lower(trigger.p) }}",
            "{{ trim(trigger.p) }}",
            "{{ json(trigger.p) }}",
            "text {{ trigger.p }} {{ trigger.p }}",
        ] {
            let template = parse_template(&json!(source)).unwrap();

            let refused = template
                .evaluate_within(&ctx, &mut EvalBudget::new(1000))
                .unwrap_err();

            assert_eq!(
                refused,
                ExpressionError::TooLarge { limit: 1000 },
                "{source}"
            );
        }
    }

    #[test]
    fn a_budget_is_shared_by_everything_evaluated_against_it() {
        let trigger = json!({ "p": "x".repeat(400) });
        let nodes = BTreeMap::new();
        let ctx = payload_context(&trigger, &nodes);
        let template = parse_template(&json!("{{ trigger.p }}")).unwrap();
        let mut budget = EvalBudget::new(1000);

        let first = template.evaluate_within(&ctx, &mut budget);
        let second = template.evaluate_within(&ctx, &mut budget);
        let third = template.evaluate_within(&ctx, &mut budget);

        assert!(first.is_ok() && second.is_ok());
        assert_eq!(
            third.unwrap_err(),
            ExpressionError::TooLarge { limit: 1000 }
        );
    }

    #[test]
    fn loop_paths_resolve_the_same_without_building_the_synthetic_object() {
        let nodes = BTreeMap::new();
        let item = json!({ "name": "Ada", "tags": ["a", "b"] });
        let ctx = ExpressionContext {
            trigger: None,
            nodes: &nodes,
            position: 0,
            loop_frame: Some(LoopFrame {
                item: &item,
                index: 4,
            }),
            now: fixed_now(),
        };
        let read = |source: &str| parse_template(&json!(source)).unwrap().evaluate(&ctx);

        assert_eq!(read("{{ loop.index }}").unwrap(), json!(4));
        assert_eq!(read("{{ loop.item.tags[1] }}").unwrap(), json!("b"));
        assert_eq!(
            read("{{ loop }}").unwrap(),
            json!({ "item": { "name": "Ada", "tags": ["a", "b"] }, "index": 4 })
        );
        assert!(matches!(
            read("{{ loop.other }}").unwrap_err(),
            ExpressionError::MissingPath { .. }
        ));
        assert!(matches!(
            read("{{ loop.index.deeper }}").unwrap_err(),
            ExpressionError::MissingPath { .. }
        ));
    }

    #[test]
    fn item_pairs_with_the_element_of_the_same_rank() {
        let nodes = nodes_of(&[(
            "Rechercher",
            vec![
                json!({ "client": { "nom": "Ada" } }),
                json!({ "client": { "nom": "Grace" } }),
                json!({ "client": { "nom": "Linus" } }),
            ],
        )]);
        let source = "{{ $('Rechercher').item.json.client.nom }}";

        assert_eq!(read(&at(&nodes, 0), source).unwrap(), json!("Ada"));
        assert_eq!(read(&at(&nodes, 1), source).unwrap(), json!("Grace"));
        assert_eq!(read(&at(&nodes, 2), source).unwrap(), json!("Linus"));
    }

    #[test]
    fn item_falls_back_to_the_last_element_when_the_list_is_shorter() {
        let nodes = nodes_of(&[("Court", vec![json!({ "n": 1 }), json!({ "n": 2 })])]);

        assert_eq!(
            read(&at(&nodes, 5), "{{ $('Court').item.json.n }}").unwrap(),
            json!(2)
        );
    }

    #[test]
    fn item_on_an_empty_list_is_an_error_naming_the_node() {
        let nodes = nodes_of(&[("Vide", Vec::new())]);
        let ctx = empty_context(&nodes);

        for source in [
            "{{ $('Vide').item.json.n }}",
            "{{ $('Vide').first().json.n }}",
            "{{ $('Vide').last().json.n }}",
        ] {
            let err = read(&ctx, source).unwrap_err();

            assert_eq!(
                err,
                ExpressionError::NoItem {
                    name: "Vide".to_string()
                },
                "{source}"
            );
            assert!(err.to_string().contains("Vide"));
        }
    }

    #[test]
    fn first_last_and_all_read_the_list_whatever_the_position() {
        let nodes = nodes_of(&[(
            "Liste",
            vec![json!({ "n": 1 }), json!({ "n": 2 }), json!({ "n": 3 })],
        )]);
        let ctx = at(&nodes, 1);

        assert_eq!(
            read(&ctx, "{{ $('Liste').first().json.n }}").unwrap(),
            json!(1)
        );
        assert_eq!(
            read(&ctx, "{{ $('Liste').last().json.n }}").unwrap(),
            json!(3)
        );
        assert_eq!(
            read(&ctx, "{{ $('Liste').all() }}").unwrap(),
            json!([{ "n": 1 }, { "n": 2 }, { "n": 3 }])
        );
        assert_eq!(read(&ctx, "{{ $('Liste').all()[1].n }}").unwrap(), json!(2));
        assert_eq!(
            read(&ctx, "{{ $('Liste').all()[2].json.n }}").unwrap(),
            json!(3)
        );
        assert_eq!(read(&ctx, "{{ len($('Liste').all()) }}").unwrap(), json!(3));
    }

    #[test]
    fn json_is_an_alias_of_the_element_and_is_optional() {
        let nodes = nodes_of(&[("Devis", vec![json!({ "client": { "nom": "Ada" } })])]);
        let ctx = empty_context(&nodes);

        for source in [
            "{{ $('Devis').item.json.client.nom }}",
            "{{ $('Devis').item.client.nom }}",
            "{{ $('Devis').json.client.nom }}",
            "{{ $('Devis').first().json.client.nom }}",
        ] {
            assert_eq!(read(&ctx, source).unwrap(), json!("Ada"), "{source}");
        }
        assert_eq!(
            read(&ctx, "{{ $('Devis').item.json }}").unwrap(),
            json!({ "client": { "nom": "Ada" } })
        );
        assert_eq!(
            read(&ctx, "{{ $('Devis').item }}").unwrap(),
            json!({ "client": { "nom": "Ada" } })
        );
    }

    #[test]
    fn a_bracketed_key_reads_a_field_that_is_not_an_identifier() {
        let nodes = nodes_of(&[(
            "Appel",
            vec![
                json!({ "headers": { "content-type": "text/plain" }, "lignes": [{ "code": "A1" }] }),
            ],
        )]);
        let ctx = empty_context(&nodes);
        let trigger = json!({ "a b": 3 });
        let with_trigger = ExpressionContext {
            trigger: Some(&trigger),
            ..empty_context(&nodes)
        };

        assert_eq!(
            read(&ctx, "{{ $('Appel').item.json.headers[\"content-type\"] }}").unwrap(),
            json!("text/plain")
        );
        assert_eq!(
            read(&ctx, "{{ $('Appel').item.json.lignes[0].code }}").unwrap(),
            json!("A1")
        );
        assert_eq!(
            read(&with_trigger, "{{ trigger[\"a b\"] }}").unwrap(),
            json!(3)
        );
    }

    #[test]
    fn a_node_name_keeps_its_spaces_accents_and_escaped_quotes() {
        let nodes = nodes_of(&[
            ("Prix d'achat", vec![json!({ "n": 1 })]),
            ("Appel l’API d’Hugo", vec![json!({ "n": 2 })]),
            ("Back\\slash", vec![json!({ "n": 3 })]),
            ("Rechercher les devis envoyés", vec![json!({ "n": 4 })]),
        ]);
        let ctx = empty_context(&nodes);

        assert_eq!(
            read(&ctx, r"{{ $('Prix d\'achat').item.json.n }}").unwrap(),
            json!(1)
        );
        assert_eq!(
            read(&ctx, "{{ $('Appel l’API d’Hugo').item.json.n }}").unwrap(),
            json!(2)
        );
        assert_eq!(
            read(&ctx, r"{{ $('Back\\slash').item.json.n }}").unwrap(),
            json!(3)
        );
        assert_eq!(
            read(
                &ctx,
                "{{ $( 'Rechercher les devis envoyés' ).item.json.n }}"
            )
            .unwrap(),
            json!(4)
        );
    }

    #[test]
    fn a_node_name_is_compared_exactly_after_trimming() {
        let nodes = nodes_of(&[("Devis", vec![json!({ "n": 1 })])]);
        let ctx = empty_context(&nodes);

        assert_eq!(
            read(&ctx, "{{ $('  Devis ').item.json.n }}").unwrap(),
            json!(1)
        );
        assert_eq!(
            read(&ctx, "{{ $('devis').item.json.n }}").unwrap_err(),
            ExpressionError::UnknownNode {
                name: "devis".to_string()
            }
        );
    }

    #[test]
    fn a_node_reference_inside_a_sentence_renders_its_value() {
        let nodes = nodes_of(&[(
            "Si",
            vec![
                json!({ "numero": "DV-1", "client": { "nom": "Ada" }, "montant": 7450 }),
                json!({ "numero": "DV-2", "client": { "nom": "Grace" }, "montant": 3280 }),
            ],
        )]);
        let source = "Devis {{ $('Si').item.json.numero }} pour {{ $('Si').item.json.client.nom }} ({{ $('Si').item.json.montant }} €)";

        assert_eq!(
            read(&at(&nodes, 1), source).unwrap(),
            json!("Devis DV-2 pour Grace (3280 €)")
        );
    }

    #[test]
    fn a_node_reference_works_inside_operators_and_functions() {
        let nodes = nodes_of(&[("Devis", vec![json!({ "montant": 7450, "nom": "ada" })])]);
        let ctx = empty_context(&nodes);

        assert_eq!(
            read(&ctx, "{{ $('Devis').item.json.montant > 5000 }}").unwrap(),
            json!(true)
        );
        assert_eq!(
            read(&ctx, "{{ upper($('Devis').item.json.nom) }}").unwrap(),
            json!("ADA")
        );
    }

    #[test]
    fn the_malformed_node_references_are_syntax_errors() {
        for source in [
            "{{ $('Devis') }}",
            "{{ $('Devis').nope }}",
            "{{ $('Devis').first }}",
            "{{ $(\"Devis\").item }}",
            "{{ $(Devis).item }}",
            "{{ $('Devis' }}",
            "{{ $('Devis).item }}",
            "{{ $ }}",
        ] {
            let err = syntax_error(source);

            assert!(
                matches!(err, ExpressionError::Syntax { .. }),
                "{source}: {err:?}"
            );
        }
    }

    #[test]
    fn a_missing_node_path_is_written_as_the_expression_reads() {
        let nodes = nodes_of(&[("Prix d'achat", vec![json!({ "a": 1 })])]);
        let ctx = empty_context(&nodes);

        let err = read(
            &ctx,
            r#"{{ $('Prix d\'achat').item.json.headers["x-y"][2] }}"#,
        )
        .unwrap_err();

        assert_eq!(
            err,
            ExpressionError::MissingPath {
                path: r#"$('Prix d\'achat').item.json.headers["x-y"][2]"#.to_string()
            }
        );
    }

    #[test]
    fn default_does_not_catch_a_node_without_items() {
        let nodes = nodes_of(&[("Vide", Vec::new())]);
        let ctx = empty_context(&nodes);

        let err = read(&ctx, "{{ default($('Vide').item.json.n, 0) }}").unwrap_err();

        assert!(matches!(err, ExpressionError::NoItem { .. }), "{err:?}");
    }

    #[test]
    fn a_node_list_is_charged_to_the_budget_before_it_is_built() {
        let big = json!({ "p": "x".repeat(600) });
        let nodes = nodes_of(&[("Gros", vec![big.clone(), big.clone(), big])]);
        let ctx = empty_context(&nodes);
        let all = parse_template(&json!("{{ $('Gros').all() }}")).unwrap();
        let sentence = parse_template(&json!("tout : {{ $('Gros').all() }}")).unwrap();
        let one = parse_template(&json!("{{ $('Gros').item.json.p }}")).unwrap();

        assert_eq!(
            all.evaluate_within(&ctx, &mut EvalBudget::new(1000))
                .unwrap_err(),
            ExpressionError::TooLarge { limit: 1000 }
        );
        assert_eq!(
            sentence
                .evaluate_within(&ctx, &mut EvalBudget::new(1000))
                .unwrap_err(),
            ExpressionError::TooLarge { limit: 1000 }
        );
        assert!(
            one.evaluate_within(&ctx, &mut EvalBudget::new(1000))
                .is_ok()
        );
    }

    #[test]
    fn the_loop_can_hand_its_item_to_a_node_lookup_expression() {
        let item = json!({ "name": "Ada" });
        let nodes = nodes_of(&[("Devis", vec![json!({ "n": 1 })])]);
        let ctx = ExpressionContext {
            loop_frame: Some(LoopFrame {
                item: &item,
                index: 0,
            }),
            ..empty_context(&nodes)
        };

        assert_eq!(
            read(&ctx, "{{ concat(loop.item.name, $('Devis').item.json.n) }}").unwrap(),
            json!("Ada1")
        );
    }
}
