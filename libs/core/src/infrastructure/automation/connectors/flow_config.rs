use serde_json::{Map, Value};

use crate::domain::automation::connector::ConnectorDescriptor;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome, DryRunInput};

pub struct FlowConfigConnector;

fn decide(config: &Map<String, Value>) -> ConnectorOutcome {
    match config.get("variables") {
        Some(Value::Object(variables)) => {
            ConnectorOutcome::Produced(Value::Object(variables.clone()))
        }
        Some(other) => ConnectorOutcome::Failed {
            error: format!(
                "`variables` must resolve to a JSON object, got {}",
                value_kind(other)
            ),
        },
        None => ConnectorOutcome::Failed {
            error: "`variables` is required".to_string(),
        },
    }
}

impl Connector for FlowConfigConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        decide(input.config)
    }

    fn dry_run(_descriptor: &ConnectorDescriptor, input: &DryRunInput<'_>) -> ConnectorOutcome {
        decide(input.config)
    }
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

#[cfg(test)]
mod tests {
    use common::OrganizationId;
    use serde_json::{Value, json};
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
    async fn a_variables_object_is_produced_as_output() {
        let mut config = serde_json::Map::new();
        config.insert(
            "variables".to_string(),
            json!({ "customer_id": "c-1", "retry_limit": 3 }),
        );

        let outcome = FlowConfigConnector.execute(input(&config)).await;

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(json!({ "customer_id": "c-1", "retry_limit": 3 }))
        );
    }

    #[tokio::test]
    async fn a_missing_variables_field_fails() {
        let config = serde_json::Map::new();

        let outcome = FlowConfigConnector.execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn a_string_variables_field_fails() {
        let mut config = serde_json::Map::new();
        config.insert("variables".to_string(), json!("not an object"));

        let outcome = FlowConfigConnector.execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn an_array_variables_field_fails() {
        let mut config = serde_json::Map::new();
        config.insert("variables".to_string(), json!(["a", "b"]));

        let outcome = FlowConfigConnector.execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn a_number_variables_field_fails() {
        let mut config = serde_json::Map::new();
        config.insert("variables".to_string(), json!(42));

        let outcome = FlowConfigConnector.execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    fn dry_run_of(config: &serde_json::Map<String, Value>) -> ConnectorOutcome {
        let descriptor = crate::domain::automation::connector::connector_catalogue()
            .get("flow.config", 1)
            .expect("flow.config is described")
            .clone();
        FlowConfigConnector::dry_run(
            &descriptor,
            &crate::domain::automation::run::DryRunInput {
                config,
                stored: None,
            },
        )
    }

    #[test]
    fn a_dry_run_produces_the_variables_the_node_holds() {
        let mut config = serde_json::Map::new();
        config.insert("variables".to_string(), json!({ "customer_id": "c-1" }));

        assert_eq!(
            dry_run_of(&config),
            ConnectorOutcome::Produced(json!({ "customer_id": "c-1" }))
        );
    }

    #[test]
    fn a_dry_run_refuses_variables_that_are_not_an_object_like_a_run_does() {
        let mut config = serde_json::Map::new();
        config.insert("variables".to_string(), json!(42));

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
