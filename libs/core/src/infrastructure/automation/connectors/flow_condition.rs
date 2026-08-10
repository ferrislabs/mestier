use serde_json::json;

use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::domain::automation::workflow::Branch;

/// `flow.condition`: reads its resolved `predicate` field and tells the
/// engine which branch to take. Pure and side-effect free — see the
/// descriptor in `domain::automation::connector::catalogue`.
pub struct FlowConditionConnector;

impl Connector for FlowConditionConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        match input.config.get("predicate") {
            Some(serde_json::Value::Bool(matched)) => {
                let taken = if *matched { Branch::Then } else { Branch::Else };
                ConnectorOutcome::Branch {
                    taken,
                    output: json!({ "matched": matched }),
                }
            }
            Some(_) => ConnectorOutcome::Failed {
                error: "`predicate` must resolve to a boolean".to_string(),
            },
            None => ConnectorOutcome::Failed {
                error: "`predicate` is required".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use common::OrganizationId;
    use serde_json::Value;
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
    async fn a_true_predicate_takes_then() {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!(true));

        let outcome = FlowConditionConnector.execute(input(&config)).await;

        assert_eq!(
            outcome,
            ConnectorOutcome::Branch {
                taken: Branch::Then,
                output: json!({ "matched": true })
            }
        );
    }

    #[tokio::test]
    async fn a_false_predicate_takes_else() {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!(false));

        let outcome = FlowConditionConnector.execute(input(&config)).await;

        assert_eq!(
            outcome,
            ConnectorOutcome::Branch {
                taken: Branch::Else,
                output: json!({ "matched": false })
            }
        );
    }

    #[tokio::test]
    async fn a_missing_predicate_fails() {
        let config = serde_json::Map::new();

        let outcome = FlowConditionConnector.execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn a_non_boolean_predicate_fails() {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!("not a bool"));

        let outcome = FlowConditionConnector.execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }
}
