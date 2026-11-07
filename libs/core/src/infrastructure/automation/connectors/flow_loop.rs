use serde_json::{Map, Value};

use crate::domain::automation::connector::ConnectorDescriptor;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome, DryRunInput};

/// `flow.loop`: reads its resolved `items` field and hands the engine the
/// list to iterate. Pure and side-effect free — see the descriptor in
/// `domain::automation::connector::catalogue`.
pub struct FlowLoopConnector;

fn decide(config: &Map<String, Value>) -> ConnectorOutcome {
    match config.get("items") {
        Some(Value::Array(items)) => ConnectorOutcome::Iterate {
            items: items.clone(),
        },
        Some(_) => ConnectorOutcome::Failed {
            error: "`items` must resolve to an array".to_string(),
        },
        None => ConnectorOutcome::Failed {
            error: "`items` is required".to_string(),
        },
    }
}

impl Connector for FlowLoopConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        decide(input.config)
    }

    fn dry_run(_descriptor: &ConnectorDescriptor, input: &DryRunInput<'_>) -> ConnectorOutcome {
        decide(input.config)
    }
}

#[cfg(test)]
mod tests {
    use common::OrganizationId;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn input(config: &serde_json::Map<String, Value>) -> ConnectorInput<'_> {
        ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config,
            credential_id: None,
            operation: None,
        }
    }

    #[tokio::test]
    async fn an_array_of_items_is_iterated_in_order() {
        let mut config = serde_json::Map::new();
        config.insert("items".to_string(), json!(["a", "b", "c"]));

        let outcome = FlowLoopConnector.execute(input(&config)).await;

        assert_eq!(
            outcome,
            ConnectorOutcome::Iterate {
                items: vec![json!("a"), json!("b"), json!("c")]
            }
        );
    }

    #[tokio::test]
    async fn an_empty_array_iterates_zero_times() {
        let mut config = serde_json::Map::new();
        config.insert("items".to_string(), json!([]));

        let outcome = FlowLoopConnector.execute(input(&config)).await;

        assert_eq!(outcome, ConnectorOutcome::Iterate { items: Vec::new() });
    }

    #[tokio::test]
    async fn a_missing_items_field_fails() {
        let config = serde_json::Map::new();

        let outcome = FlowLoopConnector.execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn a_non_array_items_field_fails() {
        let mut config = serde_json::Map::new();
        config.insert("items".to_string(), json!("not an array"));

        let outcome = FlowLoopConnector.execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    fn dry_run_of(config: &serde_json::Map<String, Value>) -> ConnectorOutcome {
        let descriptor = crate::domain::automation::connector::connector_catalogue()
            .get("flow.loop", 1)
            .expect("flow.loop is described")
            .clone();
        FlowLoopConnector::dry_run(
            &descriptor,
            &crate::domain::automation::run::DryRunInput {
                config,
                stored: None,
            },
        )
    }

    #[test]
    fn a_dry_run_hands_over_the_resolved_items_untouched() {
        let mut config = serde_json::Map::new();
        config.insert("items".to_string(), json!(["a", "b", "c", "d"]));

        assert_eq!(
            dry_run_of(&config),
            ConnectorOutcome::Iterate {
                items: vec![json!("a"), json!("b"), json!("c"), json!("d")]
            }
        );
    }

    #[test]
    fn a_dry_run_refuses_items_that_are_not_an_array_like_a_run_does() {
        let mut config = serde_json::Map::new();
        config.insert("items".to_string(), json!("not an array"));

        assert!(matches!(
            dry_run_of(&config),
            ConnectorOutcome::Failed { .. }
        ));
        assert!(matches!(
            dry_run_of(&serde_json::Map::new()),
            ConnectorOutcome::Failed { .. }
        ));
    }
}
