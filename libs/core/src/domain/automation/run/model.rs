//! The run and its steps: the durable, resumable record of one workflow
//! execution. See the module doc on [`super`] for the shape this mirrors.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::OrganizationId;
use serde_json::Value;
use uuid::Uuid;

/// A workflow run. `workflow_version_id` is pinned at creation and never
/// moved, so editing the workflow while this run is executing changes
/// nothing about what it runs — see
/// [`super::ports::RunRepository::create_run`] (declared in
/// `domain::automation::ports`).
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub id: Uuid,
    pub org_id: OrganizationId,
    /// Denormalized alongside `workflow_version_id`: #201 needs it to
    /// compare on a primary-key join.
    pub workflow_id: Uuid,
    pub workflow_version_id: Uuid,
    pub trigger_event_id: Option<Uuid>,
    /// What started the run, read into `ExpressionContext.trigger`. `None`
    /// once #201 dispatches a run from a domain event: the payload then
    /// lives on `automation.event`, reached through `trigger_event_id`.
    pub trigger_payload: Option<Value>,
    pub status: RunStatus,
    pub error: Option<String>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub locked_at: Option<DateTime<Utc>>,
    pub locked_by: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RunStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(format!("invalid run status `{other}`")),
        }
    }
}

/// One connector invocation, durably recorded before it is known to have
/// succeeded — see the module doc on [`super`] for why that is at-least-once,
/// not exactly-once. `iteration_path` is empty outside a loop; see
/// [`super::graph::format_iteration_path`].
#[derive(Debug, Clone, PartialEq)]
pub struct RunStep {
    pub id: Uuid,
    pub run_id: Uuid,
    pub connector_id: String,
    pub iteration_path: String,
    pub attempts: u32,
    pub status: StepStatus,
    pub input: Option<Value>,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    InFlight,
    Succeeded,
    Failed,
    Skipped,
    Dead,
}

impl StepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InFlight => "in_flight",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
            Self::Dead => "dead",
        }
    }

    /// A step in one of these statuses carries a trustworthy `output` (or,
    /// for `Skipped`, deliberately none) that a resumed run reads back
    /// instead of recomputing. `Failed` and `InFlight` are not: both are
    /// re-attempted — see the module doc on at-least-once execution.
    pub fn is_settled(self) -> bool {
        matches!(self, Self::Succeeded | Self::Skipped | Self::Dead)
    }
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for StepStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "in_flight" => Ok(Self::InFlight),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            "dead" => Ok(Self::Dead),
            other => Err(format!("invalid run step status `{other}`")),
        }
    }
}

/// A run claimed and ready to be worked — what
/// [`super::ports::RunRepository::claim_due`] returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueRun {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub workflow_id: Uuid,
    pub workflow_version_id: Uuid,
    pub trigger_event_id: Option<Uuid>,
    pub trigger_payload: Option<Value>,
}

/// The run-level outcome of one slice, applied by
/// [`super::ports::RunRepository::settle_run`].
#[derive(Debug, Clone, PartialEq)]
pub enum RunSettlement {
    Succeeded,
    /// A step exhausted its retry schedule: the run cannot progress any
    /// further and is terminally failed.
    Failed {
        error: String,
    },
    /// Not finished — either a step failed but is still retryable, or the
    /// slice's step/time budget ran out. Either way the run stays claimable
    /// again at `next_attempt_at`.
    Rescheduled {
        next_attempt_at: DateTime<Utc>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_status_round_trips_through_its_string_form() {
        for status in [
            RunStatus::Pending,
            RunStatus::Running,
            RunStatus::Succeeded,
            RunStatus::Failed,
            RunStatus::Cancelled,
        ] {
            assert_eq!(status.as_str().parse::<RunStatus>(), Ok(status));
        }
    }

    #[test]
    fn an_unknown_run_status_string_is_rejected() {
        assert!("not_a_status".parse::<RunStatus>().is_err());
    }

    #[test]
    fn step_status_round_trips_through_its_string_form() {
        for status in [
            StepStatus::Pending,
            StepStatus::InFlight,
            StepStatus::Succeeded,
            StepStatus::Failed,
            StepStatus::Skipped,
            StepStatus::Dead,
        ] {
            assert_eq!(status.as_str().parse::<StepStatus>(), Ok(status));
        }
    }

    #[test]
    fn an_unknown_step_status_string_is_rejected() {
        assert!("not_a_status".parse::<StepStatus>().is_err());
    }

    /// `is_settled` is what tells the engine whether a persisted step can be
    /// read back without re-executing. Only a step that finished on its own
    /// terms (succeeded, was skipped, or is permanently dead) qualifies —
    /// `Failed` (retryable) and `InFlight` (a crashed attempt) must not.
    #[test]
    fn only_a_terminal_outcome_is_settled() {
        assert!(StepStatus::Succeeded.is_settled());
        assert!(StepStatus::Skipped.is_settled());
        assert!(StepStatus::Dead.is_settled());
        assert!(!StepStatus::Pending.is_settled());
        assert!(!StepStatus::InFlight.is_settled());
        assert!(!StepStatus::Failed.is_settled());
    }
}
