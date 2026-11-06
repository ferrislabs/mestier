use common::OrganizationId;
use serde_json::{Map, Value};
use uuid::Uuid;

use super::graph::{
    ContractKey, ContractKind, OnInvalid, PlacedTrigger, TriggerKind, WebhookMethod,
};

pub const WEBHOOK_PREFIX: &str = "webhooks";
pub const MAX_WEBHOOK_PATH_LENGTH: usize = 200;
pub const MAX_WEBHOOK_BODY_BYTES: usize = 1024 * 1024;
pub const MAX_WEBHOOK_ROUTES_PER_ORGANIZATION: usize = 200;
pub const WEBHOOK_CREDENTIAL_KINDS: [&str; 3] = ["http_header", "http_basic", "bearer_token"];

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedWebhook {
    pub org_id: OrganizationId,
    pub workflow_id: Uuid,
    pub trigger_id: String,
    pub path: String,
    pub method: WebhookMethod,
    pub credential_id: Option<Uuid>,
    pub contract: Vec<ContractKey>,
    pub on_invalid: OnInvalid,
}

impl ResolvedWebhook {
    pub fn from_trigger(
        org_id: OrganizationId,
        workflow_id: Uuid,
        trigger: PlacedTrigger,
    ) -> Option<Self> {
        let TriggerKind::Webhook {
            method,
            path,
            credential_id,
            contract,
            on_invalid,
        } = trigger.kind
        else {
            return None;
        };
        Some(Self {
            org_id,
            workflow_id,
            trigger_id: trigger.id,
            path,
            method,
            credential_id,
            contract,
            on_invalid,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WebhookPathIssue {
    #[error("the path is empty")]
    Empty,
    #[error("the path is longer than {MAX_WEBHOOK_PATH_LENGTH} characters")]
    TooLong,
    #[error("the path must not start with `/{WEBHOOK_PREFIX}/`, which is added for you")]
    Prefixed,
    #[error("the path must not contain a `..` segment")]
    ParentSegment,
    #[error("the path must not contain an empty segment")]
    EmptySegment,
    #[error("the path only allows lowercase letters, digits, `-`, `_` and `/`")]
    InvalidCharacters,
}

pub fn check_webhook_path(path: &str) -> Result<(), WebhookPathIssue> {
    if path.is_empty() {
        return Err(WebhookPathIssue::Empty);
    }
    if path.chars().count() > MAX_WEBHOOK_PATH_LENGTH {
        return Err(WebhookPathIssue::TooLong);
    }
    let unrooted = path.strip_prefix('/').unwrap_or(path);
    let keeps_the_prefix = unrooted == WEBHOOK_PREFIX
        || unrooted
            .strip_prefix(WEBHOOK_PREFIX)
            .is_some_and(|rest| rest.starts_with('/'));
    if keeps_the_prefix {
        return Err(WebhookPathIssue::Prefixed);
    }
    if path.split('/').any(|segment| segment == "..") {
        return Err(WebhookPathIssue::ParentSegment);
    }
    if path.split('/').any(str::is_empty) {
        return Err(WebhookPathIssue::EmptySegment);
    }
    let allowed = |character: char| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '-' | '_' | '/')
    };
    if !path.chars().all(allowed) {
        return Err(WebhookPathIssue::InvalidCharacters);
    }
    Ok(())
}

pub fn holds_nul(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains('\0'),
        Value::Array(items) => items.iter().any(holds_nul),
        Value::Object(fields) => fields
            .iter()
            .any(|(key, item)| key.contains('\0') || holds_nul(item)),
        _ => false,
    }
}

pub fn contract_violations(contract: &[ContractKey], body: &Value) -> Vec<String> {
    let object = body.as_object();
    contract
        .iter()
        .filter(|key| {
            let present = object
                .and_then(|fields| fields.get(&key.name))
                .filter(|value| !value.is_null());
            match present {
                None => key.required,
                Some(value) => !has_kind(key.kind, value),
            }
        })
        .map(|key| key.name.clone())
        .collect()
}

fn has_kind(kind: ContractKind, value: &Value) -> bool {
    match kind {
        ContractKind::Text => value.is_string(),
        ContractKind::Number => value.is_number(),
        ContractKind::Boolean => value.is_boolean(),
        ContractKind::Object => value.is_object(),
        ContractKind::List => value.is_array(),
    }
}

pub fn sample_body(contract: &[ContractKey]) -> Value {
    Value::Object(
        contract
            .iter()
            .map(|key| (key.name.clone(), sample_value(key)))
            .collect::<Map<String, Value>>(),
    )
}

fn sample_value(key: &ContractKey) -> Value {
    let raw = key.example.trim();
    let parsed = serde_json::from_str::<Value>(raw)
        .ok()
        .filter(|value| has_kind(key.kind, value));
    match (key.kind, parsed) {
        (ContractKind::Text, _) if !raw.is_empty() => Value::String(raw.to_owned()),
        (ContractKind::Text, _) => Value::String("texte".to_owned()),
        (_, Some(value)) => value,
        (ContractKind::Number, None) => serde_json::json!(12.5),
        (ContractKind::Boolean, None) => Value::Bool(true),
        (ContractKind::Object, None) => Value::Object(Map::new()),
        (ContractKind::List, None) => Value::Array(Vec::new()),
    }
}

pub fn is_json_content_type(value: &str) -> bool {
    let essence = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    essence == "application/json"
        || (essence.starts_with("application/")
            && essence.ends_with("+json")
            && essence.len() > "application/+json".len())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn key(name: &str, kind: ContractKind, required: bool, example: &str) -> ContractKey {
        ContractKey {
            name: name.to_string(),
            kind,
            required,
            example: example.to_string(),
        }
    }

    #[test]
    fn a_path_of_lowercase_words_and_slashes_is_accepted() {
        for path in [
            "paiement-recu",
            "a",
            "a/b/c",
            "factures_2026/payees",
            "0",
            "webhooks-x",
            "x/webhooks/y",
            "a-b_c/d-e",
        ] {
            assert_eq!(check_webhook_path(path), Ok(()), "`{path}`");
        }
    }

    #[test]
    fn an_empty_path_is_refused() {
        assert_eq!(check_webhook_path(""), Err(WebhookPathIssue::Empty));
    }

    #[test]
    fn a_path_keeping_the_imposed_prefix_is_refused() {
        for path in ["webhooks", "webhooks/x", "/webhooks/x", "/webhooks"] {
            assert_eq!(
                check_webhook_path(path),
                Err(WebhookPathIssue::Prefixed),
                "`{path}`"
            );
        }
    }

    #[test]
    fn a_parent_segment_is_refused() {
        for path in ["..", "a/..", "../a", "a/../b"] {
            assert_eq!(
                check_webhook_path(path),
                Err(WebhookPathIssue::ParentSegment),
                "`{path}`"
            );
        }
    }

    #[test]
    fn an_empty_segment_is_refused() {
        for path in ["/", "/a", "a/", "a//b", "//"] {
            assert_eq!(
                check_webhook_path(path),
                Err(WebhookPathIssue::EmptySegment),
                "`{path}`"
            );
        }
    }

    #[test]
    fn anything_but_lowercase_letters_digits_dash_underscore_and_slash_is_refused() {
        for path in [
            "A", "a b", "a.b", "é", "a?b", "a#b", "a%2Fb", "a\\b", "a\nb", "a\u{0}b", "a:b", "a*",
            "a b/c", ".", "a/.", "~a", "a@b",
        ] {
            assert_eq!(
                check_webhook_path(path),
                Err(WebhookPathIssue::InvalidCharacters),
                "`{path}`"
            );
        }
    }

    #[test]
    fn the_length_is_bounded() {
        let longest = "a".repeat(MAX_WEBHOOK_PATH_LENGTH);
        let too_long = "a".repeat(MAX_WEBHOOK_PATH_LENGTH + 1);

        assert_eq!(check_webhook_path(&longest), Ok(()));
        assert_eq!(
            check_webhook_path(&too_long),
            Err(WebhookPathIssue::TooLong)
        );
    }

    #[test]
    fn the_length_counts_characters_not_bytes() {
        let wide = "é".repeat(MAX_WEBHOOK_PATH_LENGTH);

        assert_eq!(
            check_webhook_path(&wide),
            Err(WebhookPathIssue::InvalidCharacters)
        );
    }

    #[test]
    fn a_body_that_meets_the_contract_has_no_violation() {
        let contract = vec![
            key("evenement", ContractKind::Text, true, ""),
            key("montant", ContractKind::Number, true, ""),
            key("paye", ContractKind::Boolean, true, ""),
            key("client", ContractKind::Object, true, ""),
            key("lignes", ContractKind::List, true, ""),
        ];
        let body = json!({
            "evenement": "paiement.recu",
            "montant": 2340.5,
            "paye": false,
            "client": { "nom": "Marc" },
            "lignes": [],
            "extra": "ignored"
        });

        assert!(contract_violations(&contract, &body).is_empty());
    }

    #[test]
    fn a_missing_required_key_is_a_violation() {
        let contract = vec![
            key("a", ContractKind::Text, true, ""),
            key("b", ContractKind::Text, true, ""),
        ];

        assert_eq!(
            contract_violations(&contract, &json!({ "a": "x" })),
            vec!["b".to_string()]
        );
    }

    #[test]
    fn a_null_required_key_counts_as_missing() {
        let contract = vec![key("a", ContractKind::Text, true, "")];

        assert_eq!(
            contract_violations(&contract, &json!({ "a": null })),
            vec!["a".to_string()]
        );
    }

    #[test]
    fn a_missing_or_null_optional_key_is_fine() {
        let contract = vec![key("a", ContractKind::Number, false, "")];

        assert!(contract_violations(&contract, &json!({})).is_empty());
        assert!(contract_violations(&contract, &json!({ "a": null })).is_empty());
    }

    #[test]
    fn a_key_of_the_wrong_type_is_a_violation_even_when_optional() {
        let contract = vec![
            key("text", ContractKind::Text, true, ""),
            key("number", ContractKind::Number, false, ""),
            key("boolean", ContractKind::Boolean, true, ""),
            key("object", ContractKind::Object, true, ""),
            key("list", ContractKind::List, true, ""),
        ];
        let body = json!({
            "text": 1,
            "number": "1",
            "boolean": "true",
            "object": [],
            "list": {}
        });

        assert_eq!(
            contract_violations(&contract, &body),
            vec!["text", "number", "boolean", "object", "list"]
        );
    }

    #[test]
    fn the_violations_follow_the_order_of_the_contract() {
        let contract = vec![
            key("z", ContractKind::Text, true, ""),
            key("a", ContractKind::Text, true, ""),
            key("m", ContractKind::Text, true, ""),
        ];

        assert_eq!(
            contract_violations(&contract, &json!({})),
            vec!["z", "a", "m"]
        );
    }

    #[test]
    fn a_body_that_is_not_an_object_misses_every_required_key() {
        let contract = vec![
            key("a", ContractKind::Text, true, ""),
            key("b", ContractKind::Text, false, ""),
        ];

        for body in [
            json!([1]),
            json!("text"),
            json!(3),
            json!(null),
            json!(true),
        ] {
            assert_eq!(contract_violations(&contract, &body), vec!["a"], "{body}");
        }
    }

    #[test]
    fn an_empty_contract_accepts_any_body() {
        for body in [json!({}), json!([1]), json!("text"), json!(null)] {
            assert!(contract_violations(&[], &body).is_empty(), "{body}");
        }
    }

    #[test]
    fn an_integer_and_a_float_are_both_numbers() {
        let contract = vec![key("n", ContractKind::Number, true, "")];

        assert!(contract_violations(&contract, &json!({ "n": 1 })).is_empty());
        assert!(contract_violations(&contract, &json!({ "n": -1.5 })).is_empty());
    }

    #[test]
    fn the_sample_body_reads_the_example_of_each_key_by_its_type() {
        let contract = vec![
            key("evenement", ContractKind::Text, true, "paiement.recu"),
            key("montant", ContractKind::Number, true, "2340"),
            key("paye", ContractKind::Boolean, true, "false"),
            key("client", ContractKind::Object, false, r#"{"nom":"Marc"}"#),
            key("lignes", ContractKind::List, false, "[1,2]"),
        ];

        assert_eq!(
            sample_body(&contract),
            json!({
                "evenement": "paiement.recu",
                "montant": 2340,
                "paye": false,
                "client": { "nom": "Marc" },
                "lignes": [1, 2]
            })
        );
    }

    #[test]
    fn a_key_without_a_usable_example_falls_back_to_a_value_of_its_type() {
        let contract = vec![
            key("text", ContractKind::Text, true, ""),
            key("number", ContractKind::Number, true, "pas un nombre"),
            key("boolean", ContractKind::Boolean, true, "peut-etre"),
            key("object", ContractKind::Object, true, "[1]"),
            key("list", ContractKind::List, true, "{}"),
        ];

        assert_eq!(
            sample_body(&contract),
            json!({
                "text": "texte",
                "number": 12.5,
                "boolean": true,
                "object": {},
                "list": []
            })
        );
    }

    #[test]
    fn the_sample_body_of_an_empty_contract_is_an_empty_object() {
        assert_eq!(sample_body(&[]), json!({}));
    }

    #[test]
    fn the_sample_body_meets_its_own_contract() {
        let contract = vec![
            key("a", ContractKind::Text, true, "x"),
            key("b", ContractKind::Number, true, ""),
            key("c", ContractKind::Boolean, false, "oops"),
            key("d", ContractKind::Object, true, "nope"),
            key("e", ContractKind::List, true, ""),
        ];

        assert!(contract_violations(&contract, &sample_body(&contract)).is_empty());
    }

    #[test]
    fn json_content_types_are_recognised_with_their_parameters_and_suffixes() {
        for value in [
            "application/json",
            "application/json; charset=utf-8",
            "Application/JSON",
            "application/vnd.api+json",
            "application/problem+json; charset=utf-8",
            "  application/json  ",
        ] {
            assert!(is_json_content_type(value), "`{value}`");
        }
    }

    #[test]
    fn other_content_types_are_not_json() {
        for value in [
            "",
            "text/plain",
            "application/x-www-form-urlencoded",
            "multipart/form-data; boundary=x",
            "application/jsonx",
            "text/json",
            "application/xml",
            "json",
            "application/",
        ] {
            assert!(!is_json_content_type(value), "`{value}`");
        }
    }

    #[test]
    fn the_body_cap_is_one_mebibyte() {
        assert_eq!(MAX_WEBHOOK_BODY_BYTES, 1_048_576);
    }

    #[test]
    fn a_resolved_webhook_keeps_what_the_request_needs_from_its_trigger() {
        let org_id = OrganizationId(Uuid::from_u128(1));
        let workflow_id = Uuid::from_u128(2);
        let trigger = PlacedTrigger {
            id: "t1".to_string(),
            kind: TriggerKind::Webhook {
                method: WebhookMethod::Post,
                path: "ping".to_string(),
                credential_id: Some(Uuid::from_u128(3)),
                contract: vec![key("a", ContractKind::Text, true, "")],
                on_invalid: OnInvalid::Accept,
            },
            name: None,
            description: None,
        };

        let resolved = ResolvedWebhook::from_trigger(org_id, workflow_id, trigger);

        assert_eq!(
            resolved,
            Some(ResolvedWebhook {
                org_id,
                workflow_id,
                trigger_id: "t1".to_string(),
                path: "ping".to_string(),
                method: WebhookMethod::Post,
                credential_id: Some(Uuid::from_u128(3)),
                contract: vec![key("a", ContractKind::Text, true, "")],
                on_invalid: OnInvalid::Accept,
            })
        );
    }

    #[test]
    fn a_trigger_that_is_not_a_webhook_resolves_to_nothing() {
        let trigger = PlacedTrigger {
            id: "t1".to_string(),
            kind: TriggerKind::Manual,
            name: None,
            description: None,
        };

        assert_eq!(
            ResolvedWebhook::from_trigger(OrganizationId(Uuid::from_u128(1)), Uuid::nil(), trigger),
            None
        );
    }

    #[test]
    fn a_nul_character_in_a_string_or_a_key_is_found_wherever_it_hides() {
        for body in [
            json!("a\u{0}b"),
            json!({ "a": "x\u{0}" }),
            json!({ "a\u{0}": 1 }),
            json!({ "a": { "b": [1, { "c": ["ok", "\u{0}"] }] } }),
            json!([{ "\u{0}": null }]),
        ] {
            assert!(holds_nul(&body), "{body}");
        }
    }

    #[test]
    fn a_body_without_a_nul_character_is_clean() {
        for body in [
            json!(null),
            json!(1),
            json!(true),
            json!("texte \u{e9}\u{1F600}"),
            json!({ "a": [1, "b", { "c": null }] }),
            json!({}),
            json!([]),
        ] {
            assert!(!holds_nul(&body), "{body}");
        }
    }
}
