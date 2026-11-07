use serde_json::json;

use crate::domain::automation::connector::ConnectorDescriptor;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome, DryRunInput};
use crate::domain::automation::workflow::Branch;

/// `flow.condition`: reads its resolved `predicate` field and tells the
/// engine which branch to take. Pure and side-effect free — see the
/// descriptor in `domain::automation::connector::catalogue`.
pub struct FlowConditionConnector;

fn decide(config: &serde_json::Map<String, serde_json::Value>) -> ConnectorOutcome {
    match config.get("predicate") {
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

impl Connector for FlowConditionConnector {
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
    use serde_json::Value;
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

    fn sample() -> crate::domain::automation::connector::ConnectorDescriptor {
        crate::domain::automation::connector::connector_catalogue()
            .get("flow.condition", 1)
            .expect("flow.condition is described")
            .clone()
    }

    fn dry_run_of(config: &serde_json::Map<String, Value>) -> ConnectorOutcome {
        FlowConditionConnector::dry_run(
            &sample(),
            &crate::domain::automation::run::DryRunInput {
                config,
                stored: None,
            },
        )
    }

    #[test]
    fn a_dry_run_takes_the_branch_the_resolved_predicate_selects() {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!(false));

        assert_eq!(
            dry_run_of(&config),
            ConnectorOutcome::Branch {
                taken: Branch::Else,
                output: json!({ "matched": false })
            }
        );

        config.insert("predicate".to_string(), json!(true));
        assert_eq!(
            dry_run_of(&config),
            ConnectorOutcome::Branch {
                taken: Branch::Then,
                output: json!({ "matched": true })
            }
        );
    }

    #[test]
    fn a_dry_run_refuses_a_predicate_that_is_not_a_boolean_like_a_run_does() {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!("not a bool"));

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
