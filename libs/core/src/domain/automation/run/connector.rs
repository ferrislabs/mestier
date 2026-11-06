//! The contract every executable connector kind implements. Distinct from
//! [`super::super::connector::ConnectorDescriptor`]: the descriptor is what
//! the editor shows before any run exists (#197); this is what the engine
//! calls once one does (#200).

use chrono::{DateTime, Utc};
use common::OrganizationId;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::domain::automation::collection::{Collection, Operation};
use crate::domain::automation::connector::ConnectorDescriptor;
use crate::domain::automation::workflow::{Branch, OperationRef};

/// What a connector receives: its config already **resolved** — every
/// `{{ ... }}` expression evaluated against the run's current context — and
/// enough to act (`credential_id`, `org_id`, `run_id`). A connector never
/// sees a template, only the value it evaluated to.
pub struct ConnectorInput<'a> {
    pub org_id: OrganizationId,
    pub run_id: Uuid,
    pub config: &'a Map<String, Value>,
    pub credential_id: Option<Uuid>,
    pub operation: Option<&'a OperationRef>,
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
    /// An object is one item, an array is a list of items (flattened with the
    /// results of the other items a per-item connector ran for).
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
    Wait {
        resume_at: DateTime<Utc>,
        output: Value,
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

    fn dry_run(descriptor: &ConnectorDescriptor, _input: &DryRunInput<'_>) -> ConnectorOutcome {
        ConnectorOutcome::Produced(descriptor.sample_output.clone())
    }
}

#[derive(Clone, Copy)]
pub struct StoredOperation<'a> {
    pub collection: &'a Collection,
    pub operation: &'a Operation,
}

pub struct DryRunInput<'a> {
    pub config: &'a Map<String, Value>,
    pub stored: Option<StoredOperation<'a>>,
}

pub trait DryRunDispatch {
    fn dry_run(
        &self,
        descriptor: &ConnectorDescriptor,
        input: &DryRunInput<'_>,
    ) -> Option<ConnectorOutcome>;
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
        let wait = ConnectorOutcome::Wait {
            resume_at: Utc::now(),
            output: Value::Null,
        };

        assert_ne!(produced, failed);
        assert_ne!(produced, branch);
        assert_ne!(produced, iterate);
        assert_ne!(branch, iterate);
        assert_ne!(wait, produced);
        assert_ne!(wait, iterate);
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

    struct Unexecutable;

    impl Connector for Unexecutable {
        async fn execute(&self, _input: ConnectorInput<'_>) -> ConnectorOutcome {
            panic!("a dry run must never reach execute")
        }
    }

    fn descriptor_sampling(sample: Value) -> ConnectorDescriptor {
        use crate::domain::automation::connector::ExecutionMode;

        ConnectorDescriptor {
            kind: "test.unexecutable",
            version: 1,
            family: "test",
            label: "Unexecutable",
            auth: crate::domain::automation::connector::AuthRequirement::None,
            fields: &[],
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: Value::Null,
            output_mirrors_field: None,
            sample_output: sample,
        }
    }

    #[test]
    fn the_default_dry_run_answers_with_the_descriptors_sample_and_never_executes() {
        let config = Map::new();
        let input = DryRunInput {
            config: &config,
            stored: None,
        };
        let sample = serde_json::json!({ "id": 42 });

        let outcome = Unexecutable::dry_run(&descriptor_sampling(sample.clone()), &input);

        assert_eq!(outcome, ConnectorOutcome::Produced(sample));
    }
}
