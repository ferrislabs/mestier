//! The contract every executable connector kind implements. Distinct from
//! [`super::super::connector::ConnectorDescriptor`]: the descriptor is what
//! the editor shows before any run exists (#197); this is what the engine
//! calls once one does (#200).

use common::OrganizationId;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::domain::automation::workflow::Branch;

/// What a connector receives: its config already **resolved** — every
/// `{{ ... }}` expression evaluated against the run's current context — and
/// enough to act (`credential_id`, `org_id`, `run_id`). A connector never
/// sees a template, only the value it evaluated to.
pub struct ConnectorInput<'a> {
    pub org_id: OrganizationId,
    pub run_id: Uuid,
    pub config: &'a Map<String, Value>,
    pub credential_id: Option<Uuid>,
}

/// What running a connector produced.
///
/// `Failed` is not `Err`: a connector failing is ordinary and belongs to the
/// retry schedule, exactly as documented on
/// [`super::super::ports::DeliveryHandler::deliver`] for a delivery. `Branch`
/// and `Iterate` are control flow — only `flow.condition` and `flow.loop`
/// ever produce them.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectorOutcome {
    /// Rangée sous `connectors.<id>.output` for every expression evaluated
    /// afterwards.
    Produced(Value),
    Failed {
        error: String,
    },
    Branch {
        taken: Branch,
        output: Value,
    },
    Iterate {
        items: Vec<Value>,
    },
}

/// One executable connector kind. Implemented once per `kind` in
/// `infrastructure::automation::connectors`, and checked there against the
/// catalogue by an anti-drift test: every catalogue entry has an
/// implementation, and every implementation is in the catalogue.
pub trait Connector: Send + Sync {
    fn execute<'a>(
        &'a self,
        input: ConnectorInput<'a>,
    ) -> impl Future<Output = ConnectorOutcome> + Send + 'a;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ConnectorOutcome` is compared by value in the engine's tests (a
    /// connector stub returns one, the test asserts on it), so equality has
    /// to actually distinguish every variant, not just derive without
    /// exercising it.
    #[test]
    fn outcomes_of_different_variants_are_never_equal() {
        let produced = ConnectorOutcome::Produced(Value::Null);
        let failed = ConnectorOutcome::Failed {
            error: "boom".to_string(),
        };
        let branch = ConnectorOutcome::Branch {
            taken: Branch::Then,
            output: Value::Null,
        };
        let iterate = ConnectorOutcome::Iterate { items: Vec::new() };

        assert_ne!(produced, failed);
        assert_ne!(produced, branch);
        assert_ne!(produced, iterate);
        assert_ne!(branch, iterate);
    }

    #[test]
    fn outcomes_of_the_same_variant_and_value_are_equal() {
        assert_eq!(
            ConnectorOutcome::Produced(Value::from(1)),
            ConnectorOutcome::Produced(Value::from(1))
        );
        assert_ne!(
            ConnectorOutcome::Branch {
                taken: Branch::Then,
                output: Value::Null
            },
            ConnectorOutcome::Branch {
                taken: Branch::Else,
                output: Value::Null
            }
        );
    }
}
