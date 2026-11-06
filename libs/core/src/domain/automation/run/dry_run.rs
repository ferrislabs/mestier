use common::CoreError;
use serde_json::Value;

use crate::domain::automation::workflow::GraphError;

pub const MAX_DRY_RUN_LOOP_ITEMS: usize = 3;
pub const MAX_DRY_RUN_STEPS: usize = 1_000;
pub const MAX_DRY_RUN_BYTES: usize = 16 * 1024 * 1024;

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
    pub status: DryRunStatus,
    pub input: Option<Value>,
    pub output: Option<Value>,
    pub error: Option<String>,
}

impl DryRunStep {
    pub fn succeeded(connector_id: &str, input: Option<Value>, output: Value) -> Self {
        Self {
            connector_id: connector_id.to_owned(),
            status: DryRunStatus::Succeeded,
            input,
            output: Some(output),
            error: None,
        }
    }

    pub fn failed(connector_id: &str, input: Option<Value>, error: String) -> Self {
        Self {
            connector_id: connector_id.to_owned(),
            status: DryRunStatus::Failed,
            input,
            output: None,
            error: Some(error),
        }
    }

    pub fn weight(&self) -> usize {
        self.connector_id.len()
            + self.input.as_ref().map_or(0, weight_of)
            + self.output.as_ref().map_or(0, weight_of)
            + self.error.as_ref().map_or(0, String::len)
    }

    pub fn skipped(connector_id: &str) -> Self {
        Self {
            connector_id: connector_id.to_owned(),
            status: DryRunStatus::Skipped,
            input: None,
            output: None,
            error: None,
        }
    }
}

fn weight_of(value: &Value) -> usize {
    match value {
        Value::Null | Value::Bool(_) => 4,
        Value::Number(_) => 8,
        Value::String(text) => text.len() + 2,
        Value::Array(items) => 2 + items.iter().map(weight_of).sum::<usize>(),
        Value::Object(fields) => {
            2 + fields
                .iter()
                .map(|(name, item)| name.len() + 4 + weight_of(item))
                .sum::<usize>()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DryRunError {
    #[error("{}", joined(.0))]
    InvalidGraph(Vec<GraphError>),

    #[error("trigger `{trigger_id}` is not in this graph")]
    UnknownTrigger { trigger_id: String },

    #[error("a dry run walks at most {limit} steps, and this graph needs more")]
    TooManySteps { limit: usize },

    #[error("a dry run answers with at most {limit} bytes, and this graph needs more")]
    TooLarge { limit: usize },

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
    fn a_skipped_step_carries_nothing_but_its_node() {
        let step = DryRunStep::skipped("c2");

        assert_eq!(step.connector_id, "c2");
        assert_eq!(step.status, DryRunStatus::Skipped);
        assert!(step.input.is_none() && step.output.is_none() && step.error.is_none());
    }

    #[test]
    fn a_failed_step_carries_its_error_and_no_output() {
        let step = DryRunStep::failed("c2", Some(Value::Null), "boom".to_owned());

        assert_eq!(step.status, DryRunStatus::Failed);
        assert_eq!(step.error.as_deref(), Some("boom"));
        assert!(step.output.is_none());
    }

    #[test]
    fn a_succeeded_step_carries_its_output_and_no_error() {
        let step = DryRunStep::succeeded("c1", None, Value::from(1));

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
    fn a_step_weighs_what_it_carries() {
        let light = DryRunStep::skipped("c1");
        let heavy = DryRunStep::succeeded(
            "c1",
            Some(serde_json::json!({ "name": "x".repeat(1000) })),
            Value::from(1),
        );

        assert_eq!(light.weight(), 2);
        assert!(heavy.weight() > 1000);
        assert!(heavy.weight() < 1100);
    }

    #[test]
    fn the_loop_ceiling_is_the_one_the_contract_names() {
        assert_eq!(MAX_DRY_RUN_LOOP_ITEMS, 3);
    }
}
