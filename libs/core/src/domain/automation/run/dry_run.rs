use std::time::Duration;

use common::CoreError;
use serde_json::Value;

use crate::domain::automation::expression::json_size;
use crate::domain::automation::workflow::GraphError;

pub const MAX_DRY_RUN_LOOP_ITEMS: usize = 3;
pub const MAX_DRY_RUN_STEPS: usize = 1_000;
pub const MAX_DRY_RUN_PENDING: usize = 10_000;
pub const MAX_DRY_RUN_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_DRY_RUN_NODE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_DRY_RUN_DURATION: Duration = Duration::from_secs(2);
pub const MAX_DRY_RUN_NODES: usize = 1_000;
pub const MAX_DRY_RUN_EDGES: usize = 3_000;
pub const MAX_DRY_RUN_FIELD_BYTES: usize = 64 * 1024;
pub const MAX_DRY_RUN_PAYLOAD_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DryRunStatus {
    Succeeded,
    Failed,
    Skipped,
}

impl DryRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DryRunStep {
    pub connector_id: String,
    pub iteration_path: String,
    pub status: DryRunStatus,
    pub input: Option<Value>,
    pub output: Option<Value>,
    pub error: Option<String>,
}

impl DryRunStep {
    pub fn succeeded(
        connector_id: &str,
        iteration_path: &str,
        input: Option<Value>,
        output: Value,
    ) -> Self {
        Self {
            connector_id: connector_id.to_owned(),
            iteration_path: iteration_path.to_owned(),
            status: DryRunStatus::Succeeded,
            input,
            output: Some(output),
            error: None,
        }
    }

    pub fn failed(
        connector_id: &str,
        iteration_path: &str,
        input: Option<Value>,
        error: String,
    ) -> Self {
        Self {
            connector_id: connector_id.to_owned(),
            iteration_path: iteration_path.to_owned(),
            status: DryRunStatus::Failed,
            input,
            output: None,
            error: Some(error),
        }
    }

    pub fn skipped(connector_id: &str, iteration_path: &str) -> Self {
        Self {
            connector_id: connector_id.to_owned(),
            iteration_path: iteration_path.to_owned(),
            status: DryRunStatus::Skipped,
            input: None,
            output: None,
            error: None,
        }
    }

    pub fn weight(&self) -> usize {
        self.connector_id.len()
            + self.iteration_path.len()
            + self.input.as_ref().map_or(0, json_size)
            + self.output.as_ref().map_or(0, json_size)
            + self.error.as_ref().map_or(0, String::len)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DryRunLimit {
    Nodes,
    Edges,
    Field,
    Payload,
}

impl DryRunLimit {
    pub fn message(self) -> &'static str {
        match self {
            Self::Nodes => "a dry run takes at most 1000 nodes",
            Self::Edges => "a dry run takes at most 3000 edges",
            Self::Field => "a dry run takes at most 64 KiB per config field",
            Self::Payload => "a dry run takes at most 256 KiB of trigger payload",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DryRunError {
    #[error("{}", joined(.0))]
    InvalidGraph(Vec<GraphError>),

    #[error("trigger `{trigger_id}` is not in this graph")]
    UnknownTrigger { trigger_id: String },

    #[error("{}", .0.message())]
    Oversized(DryRunLimit),

    #[error("a dry run is limited to {limit} steps, and this graph needs more")]
    TooManySteps { limit: usize },

    #[error("a dry run answers with at most {limit} bytes, and this graph needs more")]
    TooLarge { limit: usize },

    #[error("a dry run is cut after 2 seconds, and this graph needs longer")]
    TimedOut,

    #[error(transparent)]
    Core(#[from] CoreError),
}

fn joined(errors: &[GraphError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_status_renders_the_word_the_wire_uses() {
        assert_eq!(DryRunStatus::Succeeded.as_str(), "succeeded");
        assert_eq!(DryRunStatus::Failed.as_str(), "failed");
        assert_eq!(DryRunStatus::Skipped.as_str(), "skipped");
    }

    #[test]
    fn a_skipped_step_carries_nothing_but_its_node_and_its_iteration() {
        let step = DryRunStep::skipped("c2", "l1[2]");

        assert_eq!(step.connector_id, "c2");
        assert_eq!(step.iteration_path, "l1[2]");
        assert_eq!(step.status, DryRunStatus::Skipped);
        assert!(step.input.is_none() && step.output.is_none() && step.error.is_none());
    }

    #[test]
    fn a_failed_step_carries_its_error_and_no_output() {
        let step = DryRunStep::failed("c2", "", Some(Value::Null), "boom".to_owned());

        assert_eq!(step.status, DryRunStatus::Failed);
        assert_eq!(step.error.as_deref(), Some("boom"));
        assert!(step.output.is_none());
        assert_eq!(step.iteration_path, "");
    }

    #[test]
    fn a_succeeded_step_carries_its_output_and_no_error() {
        let step = DryRunStep::succeeded("c1", "", None, Value::from(1));

        assert_eq!(step.status, DryRunStatus::Succeeded);
        assert_eq!(step.output, Some(Value::from(1)));
        assert!(step.error.is_none());
    }

    #[test]
    fn an_invalid_graph_renders_every_error_it_holds() {
        let error = DryRunError::InvalidGraph(vec![
            GraphError::MissingRequiredField {
                connector_id: "c1".to_owned(),
                field: "predicate".to_owned(),
            },
            GraphError::DuplicateConnectorId {
                id: "c2".to_owned(),
            },
        ]);

        let message = error.to_string();

        assert!(
            message.contains("c1") && message.contains("predicate"),
            "{message}"
        );
        assert!(message.contains("c2"), "{message}");
    }

    #[test]
    fn a_step_weighs_the_json_it_would_serialize_to() {
        let input = serde_json::json!({ "name": "é".repeat(500), "n": [1, 2.5, null] });
        let step = DryRunStep::succeeded("c1", "l[0]", Some(input.clone()), Value::from(1));

        assert_eq!(
            step.weight(),
            2 + 4 + serde_json::to_string(&input).unwrap().len() + 1
        );
        assert_eq!(DryRunStep::skipped("c1", "").weight(), 2);
    }

    #[test]
    fn the_loop_ceiling_is_the_one_the_contract_names() {
        assert_eq!(MAX_DRY_RUN_LOOP_ITEMS, 3);
    }

    #[test]
    fn each_structural_limit_says_the_number_it_enforces() {
        assert!(
            DryRunLimit::Nodes
                .message()
                .contains(&MAX_DRY_RUN_NODES.to_string())
        );
        assert!(
            DryRunLimit::Edges
                .message()
                .contains(&MAX_DRY_RUN_EDGES.to_string())
        );
        assert!(
            DryRunLimit::Field
                .message()
                .contains(&(MAX_DRY_RUN_FIELD_BYTES / 1024).to_string())
        );
        assert!(
            DryRunLimit::Payload
                .message()
                .contains(&(MAX_DRY_RUN_PAYLOAD_BYTES / 1024).to_string())
        );
        assert_eq!(MAX_DRY_RUN_DURATION, Duration::from_secs(2));
    }
}
