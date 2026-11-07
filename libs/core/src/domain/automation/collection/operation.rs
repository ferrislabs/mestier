use std::collections::HashSet;

use serde_json::{Map, Value};

use super::model::{Collection, HttpMethod, Operation, ParameterLocation};

pub const COLLECTION_OPERATION_KIND: &str = "collection.operation";
pub const NESTED_EXPRESSION_FIELDS: [&str; 4] = ["path", "query", "headers", "body"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigStage {
    Saved,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterIssue {
    Missing {
        field: String,
    },
    Unknown {
        field: String,
    },
    InvalidValue {
        field: String,
    },
    Duplicate {
        field: String,
    },
    NotAnObject {
        field: &'static str,
        got: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltRequest {
    pub method: HttpMethod,
    pub url: String,
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RequestError {
    #[error("{}", describe_issues(.0))]
    Parameters(Vec<ParameterIssue>),
    #[error("collection `{title}` has no base url to call")]
    NoBaseUrl { title: String },
    #[error("parameter `{field}` must be a string, a number or a boolean")]
    InvalidValue { field: String },
}

fn describe_issues(issues: &[ParameterIssue]) -> String {
    issues
        .iter()
        .map(|issue| match issue {
            ParameterIssue::Missing { field } => format!("missing required parameter `{field}`"),
            ParameterIssue::Unknown { field } => {
                format!("`{field}` is not a parameter of the operation")
            }
            ParameterIssue::InvalidValue { field } => {
                format!("parameter `{field}` cannot be empty, `.` or `..`")
            }
            ParameterIssue::Duplicate { field } => {
                format!("`{field}` repeats a header already set, whatever its case")
            }
            ParameterIssue::NotAnObject { field, got } => {
                format!("`{field}` must be an object, got {got}")
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

const GROUPS: [(ParameterLocation, &str); 3] = [
    (ParameterLocation::Path, "path"),
    (ParameterLocation::Query, "query"),
    (ParameterLocation::Header, "headers"),
];

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn is_expression(value: &Value) -> bool {
    value.as_str().is_some_and(|text| text.contains("{{"))
}

pub(super) fn path_placeholders(path: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = path;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            break;
        };
        names.push(&after[..end]);
        rest = &after[end + 1..];
    }
    names
}

fn normalized(location: ParameterLocation, name: &str) -> String {
    match location {
        ParameterLocation::Header => name.to_ascii_lowercase(),
        ParameterLocation::Path | ParameterLocation::Query => name.to_owned(),
    }
}

fn declared(operation: &Operation, location: ParameterLocation) -> Vec<(&str, bool)> {
    let mut names: Vec<(&str, bool)> = operation
        .parameters
        .iter()
        .filter(|parameter| parameter.location == location)
        .map(|parameter| {
            (
                parameter.name.as_str(),
                parameter.required || location == ParameterLocation::Path,
            )
        })
        .collect();
    if location == ParameterLocation::Path {
        let mut known: HashSet<&str> = names.iter().map(|(name, _)| *name).collect();
        for placeholder in path_placeholders(&operation.path) {
            if known.insert(placeholder) {
                names.push((placeholder, true));
            }
        }
    }
    names
}

fn is_given(location: ParameterLocation, value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(text) => location != ParameterLocation::Path || !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        _ => true,
    }
}

fn is_dot_segment(value: &Value) -> bool {
    matches!(value.as_str(), Some(".") | Some(".."))
}

pub fn check_config(
    operation: &Operation,
    config: &Map<String, Value>,
    stage: ConfigStage,
) -> Vec<ParameterIssue> {
    let mut issues = Vec::new();

    for (location, key) in GROUPS {
        let provided = match config.get(key) {
            None | Some(Value::Null) => None,
            Some(Value::Object(map)) => Some(map),
            Some(value) if stage == ConfigStage::Saved && is_expression(value) => continue,
            Some(value) => {
                issues.push(ParameterIssue::NotAnObject {
                    field: key,
                    got: type_name(value),
                });
                continue;
            }
        };
        let names = declared(operation, location);
        let known: HashSet<String> = names
            .iter()
            .map(|(name, _)| normalized(location, name))
            .collect();
        let given: HashSet<String> = provided
            .into_iter()
            .flatten()
            .filter(|(_, value)| is_given(location, value))
            .map(|(name, _)| normalized(location, name))
            .collect();

        for (name, required) in &names {
            if *required && !given.contains(&normalized(location, name)) {
                issues.push(ParameterIssue::Missing {
                    field: format!("{key}.{name}"),
                });
            }
        }
        let mut seen: HashSet<String> = HashSet::new();
        for (provided_name, value) in provided.into_iter().flatten() {
            let normal = normalized(location, provided_name);
            if !known.contains(&normal) {
                issues.push(ParameterIssue::Unknown {
                    field: format!("{key}.{provided_name}"),
                });
            }
            if location == ParameterLocation::Path && is_dot_segment(value) {
                issues.push(ParameterIssue::InvalidValue {
                    field: format!("{key}.{provided_name}"),
                });
            }
            if !seen.insert(normal) {
                issues.push(ParameterIssue::Duplicate {
                    field: format!("{key}.{provided_name}"),
                });
            }
        }
    }

    let body_given = config.get("body").is_some_and(|body| !body.is_null());
    if body_given && operation.request_body.is_none() {
        issues.push(ParameterIssue::Unknown {
            field: "body".to_owned(),
        });
    }

    issues
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn encode_segment(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn substitute_path(path: &str, values: &Map<String, Value>) -> Result<String, RequestError> {
    let mut rendered = String::with_capacity(path.len());
    let mut rest = path;
    while let Some(start) = rest.find('{') {
        rendered.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            rendered.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let name = &after[..end];
        let text =
            values
                .get(name)
                .and_then(scalar_text)
                .ok_or_else(|| RequestError::InvalidValue {
                    field: format!("path.{name}"),
                })?;
        rendered.push_str(&encode_segment(&text));
        rest = &after[end + 1..];
    }
    rendered.push_str(rest);
    Ok(rendered)
}

fn pairs(
    location: ParameterLocation,
    group: &str,
    values: Option<&Map<String, Value>>,
) -> Result<Vec<(String, String)>, RequestError> {
    let mut rendered = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (name, value) in values.into_iter().flatten() {
        if location == ParameterLocation::Header && !seen.insert(name.to_ascii_lowercase()) {
            continue;
        }
        let invalid = || RequestError::InvalidValue {
            field: format!("{group}.{name}"),
        };
        match value {
            Value::Null => {}
            Value::Array(items) if location == ParameterLocation::Query => {
                for item in items {
                    rendered.push((name.clone(), scalar_text(item).ok_or_else(invalid)?));
                }
            }
            other => rendered.push((name.clone(), scalar_text(other).ok_or_else(invalid)?)),
        }
    }
    Ok(rendered)
}

pub fn build_request(
    collection: &Collection,
    operation: &Operation,
    config: &Map<String, Value>,
) -> Result<BuiltRequest, RequestError> {
    let issues = check_config(operation, config, ConfigStage::Resolved);
    if !issues.is_empty() {
        return Err(RequestError::Parameters(issues));
    }
    let base = collection.base_url.trim_end_matches('/');
    if base.is_empty() {
        return Err(RequestError::NoBaseUrl {
            title: collection.title.clone(),
        });
    }

    let path_values = config.get("path").and_then(Value::as_object);
    let empty = Map::new();
    let path = substitute_path(&operation.path, path_values.unwrap_or(&empty))?;

    Ok(BuiltRequest {
        method: operation.method,
        url: format!("{base}{path}"),
        query: pairs(
            ParameterLocation::Query,
            "query",
            config.get("query").and_then(Value::as_object),
        )?,
        headers: pairs(
            ParameterLocation::Header,
            "headers",
            config.get("headers").and_then(Value::as_object),
        )?,
        body: config.get("body").filter(|body| !body.is_null()).cloned(),
    })
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use common::OrganizationId;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::domain::automation::collection::model::{
        CollectionDraft, CollectionDraftInput, CollectionSource, Parameter, ParameterLocation,
        RequestBody,
    };

    fn parameter(name: &str, location: ParameterLocation, required: bool) -> Parameter {
        Parameter {
            name: name.to_owned(),
            location,
            required,
            description: String::new(),
            schema: json!({ "type": "string" }),
        }
    }

    fn operation(path: &str, parameters: Vec<Parameter>, with_body: bool) -> Operation {
        Operation {
            id: "op".to_owned(),
            method: HttpMethod::Post,
            path: path.to_owned(),
            summary: String::new(),
            tag: String::new(),
            parameters,
            request_body: with_body.then(|| RequestBody {
                schema: json!({ "type": "object" }),
            }),
            response: None,
            response_status: None,
        }
    }

    fn collection(base_url: &str, operation: Operation) -> Collection {
        let draft = CollectionDraft::new(CollectionDraftInput {
            title: "Things".to_owned(),
            version: "1".to_owned(),
            description: String::new(),
            base_url: base_url.to_owned(),
            source: CollectionSource::Url {
                url: "https://example.com/openapi.json".to_owned(),
            },
            security: Vec::new(),
            operations: vec![operation],
            schemas: Map::new(),
        })
        .unwrap();
        Collection::from_draft(
            draft,
            Uuid::from_u128(1),
            OrganizationId(Uuid::from_u128(2)),
            "things".to_owned(),
            Utc::now(),
        )
    }

    fn config(value: Value) -> Map<String, Value> {
        value.as_object().cloned().expect("a config is an object")
    }

    fn standard() -> Operation {
        operation(
            "/things/{id}",
            vec![
                parameter("id", ParameterLocation::Path, true),
                parameter("limit", ParameterLocation::Query, false),
                parameter("page", ParameterLocation::Query, true),
                parameter("X-Trace", ParameterLocation::Header, false),
                parameter("X-Tenant", ParameterLocation::Header, true),
            ],
            true,
        )
    }

    fn missing(field: &str) -> ParameterIssue {
        ParameterIssue::Missing {
            field: field.to_owned(),
        }
    }

    fn unknown(field: &str) -> ParameterIssue {
        ParameterIssue::Unknown {
            field: field.to_owned(),
        }
    }

    #[test]
    fn a_config_providing_every_required_parameter_has_no_issue() {
        let issues = check_config(
            &standard(),
            &config(json!({
                "path": { "id": "42" },
                "query": { "page": 1 },
                "headers": { "X-Tenant": "acme" },
                "body": { "name": "x" }
            })),
            ConfigStage::Saved,
        );

        assert_eq!(issues, Vec::new());
    }

    #[test]
    fn every_missing_required_parameter_is_reported_with_its_group() {
        let issues = check_config(&standard(), &config(json!({})), ConfigStage::Saved);

        assert_eq!(
            issues,
            vec![
                missing("path.id"),
                missing("query.page"),
                missing("headers.X-Tenant")
            ]
        );
    }

    #[test]
    fn a_null_value_counts_as_absent() {
        let issues = check_config(
            &standard(),
            &config(json!({
                "path": { "id": null },
                "query": { "page": 1 },
                "headers": { "X-Tenant": "acme" }
            })),
            ConfigStage::Saved,
        );

        assert_eq!(issues, vec![missing("path.id")]);
    }

    #[test]
    fn a_name_the_operation_does_not_declare_is_unknown_in_its_group() {
        let issues = check_config(
            &standard(),
            &config(json!({
                "path": { "id": "1", "other": "x" },
                "query": { "page": 1, "sort": "asc" },
                "headers": { "X-Tenant": "acme", "X-Other": "y" }
            })),
            ConfigStage::Saved,
        );

        assert_eq!(
            issues,
            vec![
                unknown("path.other"),
                unknown("query.sort"),
                unknown("headers.X-Other")
            ]
        );
    }

    #[test]
    fn a_header_is_matched_without_regard_to_case() {
        let issues = check_config(
            &standard(),
            &config(json!({
                "path": { "id": "1" },
                "query": { "page": 1 },
                "headers": { "x-tenant": "acme", "X-TRACE": "t" }
            })),
            ConfigStage::Saved,
        );

        assert_eq!(issues, Vec::new());
    }

    #[test]
    fn a_path_placeholder_without_a_declared_parameter_is_still_required() {
        let op = operation("/things/{code}/parts/{part}", Vec::new(), false);

        let issues = check_config(
            &op,
            &config(json!({ "path": { "code": "a" } })),
            ConfigStage::Saved,
        );

        assert_eq!(issues, vec![missing("path.part")]);
        let fine = check_config(
            &op,
            &config(json!({ "path": { "code": "a", "part": "b" } })),
            ConfigStage::Saved,
        );
        assert_eq!(fine, Vec::new());
    }

    #[test]
    fn a_body_for_an_operation_without_a_request_body_is_unknown() {
        let op = operation("/things", Vec::new(), false);

        let with_body = check_config(
            &op,
            &config(json!({ "body": { "a": 1 } })),
            ConfigStage::Saved,
        );
        let null_body = check_config(&op, &config(json!({ "body": null })), ConfigStage::Saved);
        let no_body = check_config(&op, &config(json!({})), ConfigStage::Saved);

        assert_eq!(with_body, vec![unknown("body")]);
        assert_eq!(null_body, Vec::new());
        assert_eq!(no_body, Vec::new());
    }

    #[test]
    fn a_whole_group_given_as_an_expression_is_left_alone_when_saved() {
        let issues = check_config(
            &standard(),
            &config(json!({
                "path": "{{ trigger.path }}",
                "query": "{{ trigger.query }}",
                "headers": "{{ trigger.headers }}"
            })),
            ConfigStage::Saved,
        );

        assert_eq!(issues, Vec::new());
    }

    #[test]
    fn a_group_that_is_not_an_object_is_reported_once_resolved() {
        let issues = check_config(
            &standard(),
            &config(json!({
                "path": "{{ trigger.path }}",
                "query": 5,
                "headers": [1]
            })),
            ConfigStage::Resolved,
        );

        assert_eq!(
            issues,
            vec![
                ParameterIssue::NotAnObject {
                    field: "path",
                    got: "string"
                },
                ParameterIssue::NotAnObject {
                    field: "query",
                    got: "number"
                },
                ParameterIssue::NotAnObject {
                    field: "headers",
                    got: "array"
                },
            ]
        );
    }

    #[test]
    fn a_literal_that_is_not_an_object_is_reported_even_when_saved() {
        let issues = check_config(
            &operation("/things", Vec::new(), false),
            &config(json!({ "query": 5, "headers": "plain" })),
            ConfigStage::Saved,
        );

        assert_eq!(
            issues,
            vec![
                ParameterIssue::NotAnObject {
                    field: "query",
                    got: "number"
                },
                ParameterIssue::NotAnObject {
                    field: "headers",
                    got: "string"
                },
            ]
        );
    }

    #[test]
    fn an_empty_path_value_counts_as_missing_and_dot_segments_are_invalid() {
        let op = operation(
            "/things/{id}",
            vec![parameter("id", ParameterLocation::Path, true)],
            false,
        );

        for stage in [ConfigStage::Saved, ConfigStage::Resolved] {
            let empty = check_config(&op, &config(json!({ "path": { "id": "" } })), stage);
            assert_eq!(empty, vec![missing("path.id")], "{stage:?}");

            for dots in [".", ".."] {
                let issues = check_config(&op, &config(json!({ "path": { "id": dots } })), stage);
                assert_eq!(
                    issues,
                    vec![ParameterIssue::InvalidValue {
                        field: "path.id".to_owned()
                    }],
                    "{dots} {stage:?}"
                );
            }

            for fine in ["...", "a.b", ".hidden", "..x", "a..", "0"] {
                let issues = check_config(&op, &config(json!({ "path": { "id": fine } })), stage);
                assert_eq!(issues, Vec::new(), "{fine} {stage:?}");
            }
        }
    }

    #[test]
    fn a_whole_expression_in_a_path_value_is_not_judged_before_it_runs() {
        let op = operation(
            "/things/{id}",
            vec![parameter("id", ParameterLocation::Path, true)],
            false,
        );

        let issues = check_config(
            &op,
            &config(json!({ "path": { "id": "{{ trigger.id }}" } })),
            ConfigStage::Saved,
        );

        assert_eq!(issues, Vec::new());
    }

    #[test]
    fn a_dot_segment_never_reaches_the_url() {
        let op = operation(
            "/things/{id}",
            vec![parameter("id", ParameterLocation::Path, true)],
            false,
        );
        let col = collection("https://api.example.com", op.clone());

        for dots in [".", ".."] {
            let error =
                build_request(&col, &op, &config(json!({ "path": { "id": dots } }))).unwrap_err();

            assert!(error.to_string().contains("path.id"), "{dots}: {error}");
        }
        let error = build_request(&col, &op, &config(json!({ "path": { "id": "" } }))).unwrap_err();
        assert!(error.to_string().contains("missing"), "{error}");
    }

    #[test]
    fn an_empty_array_counts_as_missing_for_a_required_query_parameter() {
        let op = operation(
            "/things",
            vec![
                parameter("tags", ParameterLocation::Query, true),
                parameter("more", ParameterLocation::Query, false),
            ],
            false,
        );

        let empty = check_config(
            &op,
            &config(json!({ "query": { "tags": [], "more": [] } })),
            ConfigStage::Resolved,
        );
        let filled = check_config(
            &op,
            &config(json!({ "query": { "tags": ["a"] } })),
            ConfigStage::Resolved,
        );

        assert_eq!(empty, vec![missing("query.tags")]);
        assert_eq!(filled, Vec::new());
    }

    #[test]
    fn headers_repeated_under_another_case_are_a_duplicate() {
        let op = operation(
            "/things",
            vec![parameter("X-Trace", ParameterLocation::Header, false)],
            false,
        );

        let issues = check_config(
            &op,
            &config(json!({ "headers": { "X-Trace": "a", "x-trace": "b" } })),
            ConfigStage::Saved,
        );

        assert_eq!(
            issues,
            vec![ParameterIssue::Duplicate {
                field: "headers.x-trace".to_owned()
            }]
        );
        assert!(
            RequestError::Parameters(issues)
                .to_string()
                .contains("headers.x-trace")
        );
    }

    #[test]
    fn rendering_headers_keeps_the_first_of_two_names_that_differ_only_by_case() {
        let values = config(json!({ "X-Trace": "first", "x-trace": "second", "X-Other": "o" }));

        let rendered = pairs(ParameterLocation::Header, "headers", Some(&values)).unwrap();

        assert_eq!(rendered.len(), 2);
        assert!(rendered.iter().any(|(_, value)| value == "o"));
        let traces: Vec<&String> = rendered
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("x-trace"))
            .map(|(_, value)| value)
            .collect();
        assert_eq!(traces.len(), 1);
    }

    #[test]
    fn a_full_hundred_parameter_operation_is_checked_in_one_pass() {
        let parameters: Vec<Parameter> = (0..100)
            .map(|n| parameter(&format!("p{n}"), ParameterLocation::Query, true))
            .collect();
        let op = operation("/things", parameters, false);
        let given: Map<String, Value> = (0..100).map(|n| (format!("p{n}"), json!(n))).collect();

        let issues = check_config(
            &op,
            &config(json!({ "query": Value::Object(given) })),
            ConfigStage::Resolved,
        );

        assert_eq!(issues, Vec::new());
    }

    #[test]
    fn the_url_is_the_base_plus_the_path_with_the_parameter_encoded() {
        let op = standard();
        let col = collection("https://api.example.com/v1/", op.clone());

        let request = build_request(
            &col,
            &op,
            &config(json!({
                "path": { "id": "a b/ç" },
                "query": { "page": 1 },
                "headers": { "X-Tenant": "acme" }
            })),
        )
        .unwrap();

        assert_eq!(
            request.url,
            "https://api.example.com/v1/things/a%20b%2F%C3%A7"
        );
        assert_eq!(request.method, HttpMethod::Post);
    }

    #[test]
    fn query_and_headers_are_rendered_as_text_and_arrays_repeat() {
        let op = operation(
            "/things",
            vec![
                parameter("a", ParameterLocation::Query, false),
                parameter("b", ParameterLocation::Query, false),
                parameter("c", ParameterLocation::Query, false),
                parameter("tags", ParameterLocation::Query, false),
                parameter("skipped", ParameterLocation::Query, false),
                parameter("X-N", ParameterLocation::Header, false),
            ],
            false,
        );
        let col = collection("https://api.example.com", op.clone());

        let request = build_request(
            &col,
            &op,
            &config(json!({
                "query": { "a": "x y", "b": 2, "c": true, "tags": ["p", 3], "skipped": null },
                "headers": { "X-N": 7 }
            })),
        )
        .unwrap();

        assert_eq!(
            request.query,
            vec![
                ("a".to_owned(), "x y".to_owned()),
                ("b".to_owned(), "2".to_owned()),
                ("c".to_owned(), "true".to_owned()),
                ("tags".to_owned(), "p".to_owned()),
                ("tags".to_owned(), "3".to_owned()),
            ]
        );
        assert_eq!(request.headers, vec![("X-N".to_owned(), "7".to_owned())]);
    }

    #[test]
    fn the_body_travels_as_given_and_a_null_body_is_no_body() {
        let op = operation("/things", Vec::new(), true);
        let col = collection("https://api.example.com", op.clone());

        let with = build_request(&col, &op, &config(json!({ "body": { "name": "x" } }))).unwrap();
        let without = build_request(&col, &op, &config(json!({ "body": null }))).unwrap();

        assert_eq!(with.body, Some(json!({ "name": "x" })));
        assert_eq!(without.body, None);
    }

    #[test]
    fn a_collection_without_a_base_url_cannot_build_a_request() {
        let op = operation("/things", Vec::new(), false);
        let col = collection("", op.clone());

        let error = build_request(&col, &op, &config(json!({}))).unwrap_err();

        assert_eq!(
            error,
            RequestError::NoBaseUrl {
                title: "Things".to_owned()
            }
        );
        assert!(error.to_string().contains("Things"));
    }

    #[test]
    fn a_missing_required_parameter_fails_the_build_and_names_it() {
        let op = standard();
        let col = collection("https://api.example.com", op.clone());

        let error = build_request(&col, &op, &config(json!({}))).unwrap_err();

        let RequestError::Parameters(issues) = &error else {
            panic!("expected parameter issues, got {error:?}");
        };
        assert!(issues.contains(&missing("path.id")));
        assert!(error.to_string().contains("path.id"), "{error}");
    }

    #[test]
    fn a_value_that_is_not_a_scalar_fails_the_build_and_names_the_field() {
        let op = operation(
            "/things/{id}",
            vec![parameter("id", ParameterLocation::Path, true)],
            false,
        );
        let col = collection("https://api.example.com", op.clone());

        let error = build_request(
            &col,
            &op,
            &config(json!({ "path": { "id": { "nested": 1 } } })),
        )
        .unwrap_err();

        assert_eq!(
            error,
            RequestError::InvalidValue {
                field: "path.id".to_owned()
            }
        );
    }

    #[test]
    fn nothing_but_the_stored_collection_decides_the_target() {
        let op = operation("/things", Vec::new(), false);
        let col = collection("https://api.example.com", op.clone());

        let request = build_request(
            &col,
            &op,
            &config(json!({
                "url": "http://169.254.169.254/latest/meta-data",
                "method": "DELETE",
                "base_url": "http://evil.example"
            })),
        )
        .unwrap();

        assert_eq!(request.url, "https://api.example.com/things");
        assert_eq!(request.method, HttpMethod::Post);
    }
}
