use serde_json::Value;

use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};

pub struct FlowConfigConnector;

impl Connector for FlowConfigConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        match input.config.get("variables") {
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
}
