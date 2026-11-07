use serde_json::{Map, Value, json};

use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};

pub struct FlowMergeConnector;

enum Mode<'a> {
    Append,
    ByPosition,
    ByField(&'a str),
}

fn failed(error: &str) -> ConnectorOutcome {
    ConnectorOutcome::Failed {
        error: error.to_owned(),
    }
}

fn mode_of(config: &Map<String, Value>) -> Result<Mode<'_>, &'static str> {
    match config.get("mode").and_then(Value::as_str) {
        Some("append") => Ok(Mode::Append),
        Some("combine_by_position") => Ok(Mode::ByPosition),
        Some("combine_by_field") => config
            .get("field")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|field| !field.is_empty())
            .map(Mode::ByField)
            .ok_or("`field` is required to combine by field"),
        _ => Err("`mode` must be append, combine_by_position or combine_by_field"),
    }
}

fn elements(value: &Value) -> Vec<&Value> {
    match value {
        Value::Null => Vec::new(),
        Value::Array(items) => items.iter().collect(),
        Value::Object(map) => match map.get("items") {
            Some(Value::Array(items)) => items.iter().collect(),
            _ => vec![value],
        },
        other => vec![other],
    }
}

fn overlay(target: &mut Map<String, Value>, source: &Map<String, Value>) {
    for (key, value) in source {
        target.insert(key.clone(), value.clone());
    }
}

fn append(lists: &[Vec<&Value>]) -> Vec<Value> {
    lists.iter().flatten().map(|item| (*item).clone()).collect()
}

fn by_position(lists: &[Vec<&Value>]) -> Vec<Value> {
    let longest = lists.iter().map(Vec::len).max().unwrap_or(0);
    (0..longest)
        .filter_map(|rank| {
            let present: Vec<&Value> = lists
                .iter()
                .filter_map(|list| list.get(rank))
                .copied()
                .collect();
            let mut objects = present
                .iter()
                .filter_map(|item| item.as_object())
                .peekable();
            if objects.peek().is_none() {
                return present.first().map(|item| (*item).clone());
            }
            let mut merged = Map::new();
            for object in objects {
                overlay(&mut merged, object);
            }
            Some(Value::Object(merged))
        })
        .collect()
}

fn by_field(lists: &[Vec<&Value>], field: &str) -> Vec<Value> {
    let Some((base, others)) = lists.split_first() else {
        return Vec::new();
    };
    base.iter()
        .filter_map(|item| item.as_object())
        .filter_map(|object| {
            let key = object.get(field).filter(|key| !key.is_null())?;
            let mut merged = object.clone();
            for other in others {
                let matching = other
                    .iter()
                    .filter_map(|item| item.as_object())
                    .find(|candidate| candidate.get(field) == Some(key))?;
                overlay(&mut merged, matching);
            }
            Some(Value::Object(merged))
        })
        .collect()
}

fn decide(config: &Map<String, Value>) -> ConnectorOutcome {
    let mode = match mode_of(config) {
        Ok(mode) => mode,
        Err(error) => return failed(error),
    };
    let inputs: &[Value] = match config.get("inputs") {
        Some(Value::Array(inputs)) => inputs,
        _ => &[],
    };
    let lists: Vec<Vec<&Value>> = inputs.iter().map(elements).collect();

    let items = match mode {
        Mode::Append => append(&lists),
        Mode::ByPosition => by_position(&lists),
        Mode::ByField(field) => by_field(&lists, field),
    };
    ConnectorOutcome::Produced(json!({ "items": items }))
}

impl Connector for FlowMergeConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        decide(input.config)
    }
}

#[cfg(test)]
mod tests {
    use common::OrganizationId;
    use uuid::Uuid;

    use super::*;
    use crate::domain::automation::connector::connector_catalogue;
    use crate::domain::automation::run::DryRunInput;

    fn config(mode: &str, field: Option<&str>, inputs: Value) -> Map<String, Value> {
        let mut config = Map::new();
        config.insert("mode".to_owned(), json!(mode));
        if let Some(field) = field {
            config.insert("field".to_owned(), json!(field));
        }
        config.insert("inputs".to_owned(), inputs);
        config
    }

    fn items(outcome: ConnectorOutcome) -> Value {
        match outcome {
            ConnectorOutcome::Produced(output) => output["items"].clone(),
            other => panic!("expected an output, got {other:?}"),
        }
    }

    fn failure(outcome: ConnectorOutcome) -> String {
        match outcome {
            ConnectorOutcome::Failed { error } => error,
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    #[test]
    fn append_concatenates_the_lists_of_every_input_in_order() {
        let inputs = json!([
            { "items": [{ "id": 1 }, { "id": 2 }] },
            [{ "id": 3 }],
            { "id": 4 },
        ]);

        let merged = items(decide(&config("append", None, inputs)));

        assert_eq!(
            merged,
            json!([{ "id": 1 }, { "id": 2 }, { "id": 3 }, { "id": 4 }])
        );
    }

    #[test]
    fn append_of_nothing_is_an_empty_list() {
        assert_eq!(items(decide(&config("append", None, json!([])))), json!([]));
        let mut without_inputs = config("append", None, json!([]));
        without_inputs.remove("inputs");
        assert_eq!(items(decide(&without_inputs)), json!([]));
    }

    #[test]
    fn append_skips_an_input_with_no_value() {
        let merged = items(decide(&config("append", None, json!([null, { "a": 1 }]))));

        assert_eq!(merged, json!([{ "a": 1 }]));
    }

    #[test]
    fn combine_by_position_merges_the_objects_of_the_same_rank() {
        let inputs = json!([
            { "items": [{ "id": 1, "name": "A" }, { "id": 2, "name": "B" }] },
            { "items": [{ "total": 10 }, { "total": 20 }, { "total": 30 }] },
        ]);

        let merged = items(decide(&config("combine_by_position", None, inputs)));

        assert_eq!(
            merged,
            json!([
                { "id": 1, "name": "A", "total": 10 },
                { "id": 2, "name": "B", "total": 20 },
                { "total": 30 },
            ])
        );
    }

    #[test]
    fn combine_by_position_lets_the_later_input_win_a_shared_key() {
        let merged = items(decide(&config(
            "combine_by_position",
            None,
            json!([{ "a": 1, "b": 1 }, { "b": 2 }]),
        )));

        assert_eq!(merged, json!([{ "a": 1, "b": 2 }]));
    }

    #[test]
    fn combine_by_position_keeps_a_lone_non_object_and_drops_one_beside_an_object() {
        let merged = items(decide(&config(
            "combine_by_position",
            None,
            json!([[1, { "a": 1 }], [2, 3]]),
        )));

        assert_eq!(merged, json!([1, { "a": 1 }]));
    }

    #[test]
    fn combine_by_field_joins_the_objects_sharing_the_field_value() {
        let inputs = json!([
            { "items": [{ "id": 1, "name": "A" }, { "id": 2, "name": "B" }, { "id": 3, "name": "C" }] },
            { "items": [{ "id": 3, "total": 30 }, { "id": 1, "total": 10 }] },
        ]);

        let merged = items(decide(&config("combine_by_field", Some("id"), inputs)));

        assert_eq!(
            merged,
            json!([
                { "id": 1, "name": "A", "total": 10 },
                { "id": 3, "name": "C", "total": 30 },
            ])
        );
    }

    #[test]
    fn combine_by_field_takes_the_first_match_and_drops_what_has_no_key() {
        let inputs = json!([
            [{ "id": 1 }, { "name": "no id" }, { "id": null }, 7],
            [{ "id": 1, "n": "first" }, { "id": 1, "n": "second" }],
        ]);

        let merged = items(decide(&config("combine_by_field", Some("id"), inputs)));

        assert_eq!(merged, json!([{ "id": 1, "n": "first" }]));
    }

    #[test]
    fn combine_by_field_needs_a_match_in_every_other_input() {
        let inputs = json!([
            [{ "id": 1 }, { "id": 2 }],
            [{ "id": 1, "a": true }],
            [{ "id": 2, "b": true }],
        ]);

        let merged = items(decide(&config("combine_by_field", Some("id"), inputs)));

        assert_eq!(merged, json!([]));
    }

    #[test]
    fn combine_by_field_without_a_field_fails_naming_it() {
        for field in [None, Some(""), Some("  ")] {
            let outcome = decide(&config("combine_by_field", field, json!([])));

            assert_eq!(failure(outcome), "`field` is required to combine by field");
        }
    }

    #[test]
    fn an_unknown_mode_fails_naming_the_field() {
        for mode in [json!("zip"), json!(3), json!(null)] {
            let mut config = config("append", None, json!([]));
            config.insert("mode".to_owned(), mode);

            assert_eq!(
                failure(decide(&config)),
                "`mode` must be append, combine_by_position or combine_by_field"
            );
        }
    }

    #[tokio::test]
    async fn executing_merges_the_inputs() {
        let config = config("append", None, json!([[1], [2]]));

        let outcome = FlowMergeConnector
            .execute(ConnectorInput {
                org_id: OrganizationId(Uuid::from_u128(1)),
                run_id: Uuid::from_u128(2),
                config: &config,
                credential_id: None,
                operation: None,
            })
            .await;

        assert_eq!(items(outcome), json!([1, 2]));
    }

    #[test]
    fn the_dry_run_answers_with_the_sample() {
        let catalogue = connector_catalogue();
        let descriptor = catalogue.get("flow.merge", 1).unwrap();
        let config = config("append", None, json!([]));

        let outcome = FlowMergeConnector::dry_run(
            descriptor,
            &DryRunInput {
                config: &config,
                stored: None,
            },
        );

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(descriptor.sample_output.clone())
        );
    }

    #[test]
    fn the_sample_has_the_shape_of_what_a_merge_outputs() {
        let ConnectorOutcome::Produced(output) =
            decide(&config("append", None, json!([{ "id": 1 }])))
        else {
            panic!("a merge produces an output");
        };

        super::super::test_support::assert_same_shape(&output, "flow.merge");
    }
}
