use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    CustomerContextId, CustomerId, MemberId, OrganizationId, ProjectId, QuoteId, TaskRecurrenceId,
};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TaskId(pub Uuid);

impl FromStr for TaskId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TaskId)
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TaskAssignmentId(pub Uuid);

impl FromStr for TaskAssignmentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TaskAssignmentId)
    }
}

impl Display for TaskAssignmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Planned,
    InProgress,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "PLANNED",
            Self::InProgress => "IN_PROGRESS",
            Self::Done => "DONE",
            Self::Cancelled => "CANCELLED",
        }
    }
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PLANNED" => Ok(Self::Planned),
            "IN_PROGRESS" => Ok(Self::InProgress),
            "DONE" => Ok(Self::Done),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(format!("invalid task status `{other}`")),
        }
    }
}

/// A reference to a plannable resource, as carried by the `PATCH` payload's
/// `assignees` list (see the planning module design doc).
///
/// One variant, one identifier. It used to be two — an employee record or an
/// organization member who might not have one — and resolving the second
/// triggered on-the-fly employee-record creation. Every member is now
/// assignable on its own, so there is nothing to provision and nothing to
/// discriminate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssigneeRef(pub MemberId);

#[derive(Debug, Clone, PartialEq)]
pub struct TaskAssignment {
    pub id: TaskAssignmentId,
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub member_id: MemberId,
    pub created_at: DateTime<Utc>,
}

/// The planning module's unit: a meeting, a trip, a training session, or a
/// "chantier" — the special case that carries a customer. See the planning
/// module design doc's vocabulary section: a chantier is not a distinct
/// entity, it is a task with `customer_id`/`customer_context_id` set.
///
/// The schema is recursive (`parent_task_id` can itself point at a task with
/// a parent), but the domain caps the hierarchy at two levels — see
/// [`service::validate_parent_depth`]. `starts_at`/`ends_at` are `None` on a
/// subtask that inherits its parent's window; [`service::resolve_task_window`]
/// resolves the effective window at read time. A root task always carries
/// its own dates (enforced by the `chk_tasks_root_has_dates` constraint and,
/// redundantly, by [`service::TaskService::create_task`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub organization_id: OrganizationId,
    pub parent_task_id: Option<TaskId>,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub status: TaskStatus,
    /// Declared at creation, never guessed: only a task that carries this as
    /// `true` produces a conflict in `detect_conflicts` — a reminder does
    /// not block anyone's availability, a meeting does.
    pub blocks_availability: bool,
    /// `customer_id`/`customer_context_id` are both present or both absent —
    /// a task with a customer *is* a chantier, nothing more.
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
    /// The project this task is costed against, when it has one. Unrelated to
    /// `parent_task_id`: a subtask attaches to a project directly, which is
    /// what the two-level hierarchy cap used to make impossible. A task with
    /// `None` costs nobody's project, and still counts towards the person who
    /// was assigned to it.
    pub project_id: Option<ProjectId>,
    /// What the task costs beyond somebody's time: travel, a skip, a parking
    /// fee. Free-form on purpose — a kilometric scale is fairer and only works
    /// if distances actually get entered (see ADR 0002). `0` means no expense,
    /// and carries no label: see [`Self::expenses_label`].
    pub expenses_cents: i32,
    /// Why the money was spent. Present exactly when `expenses_cents` is
    /// non-zero, enforced both here (`TaskService`) and by
    /// `chk_tasks_expenses_label_required`. An amount with no reason cannot be
    /// audited three months later, and clearing the amount clears the reason
    /// with it.
    pub expenses_label: Option<String>,
    /// The complete set of employees currently assigned. Always loaded and
    /// persisted as a whole — see the `PATCH` contract: `assignees` is the
    /// full list, never a delta. A subtask never inherits its parent's
    /// assignees: without one of its own, it is assigned to nobody.
    pub assignments: Vec<TaskAssignment>,
    /// The series this occurrence was materialized from, when it still
    /// follows one. `PATCH`ing any of this task's own fields sets this to
    /// `None` — see `service::TaskService::patch_task`'s detach-on-edit
    /// rule: an edited occurrence stops following the rule and keeps its
    /// own life. Never set by `TaskService::create_task`; only
    /// `TaskRecurrenceService::materialize_range` produces a task carrying
    /// this.
    pub recurrence_id: Option<TaskRecurrenceId>,
    /// The calendar date this occurrence was materialized for, in the
    /// recurrence's own timezone. Kept even after `recurrence_id` is
    /// cleared by an edit, so the occurrence's own history stays readable —
    /// it answers "which Tuesday was this" long after the task stopped
    /// following the series.
    pub occurrence_date: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = TaskId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn task_id_rejects_invalid_uuid() {
        assert!(TaskId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn task_assignment_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = TaskAssignmentId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn task_assignment_id_rejects_invalid_uuid() {
        assert!(TaskAssignmentId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn task_status_parses_known_values() {
        assert_eq!(
            "PLANNED".parse::<TaskStatus>().unwrap(),
            TaskStatus::Planned
        );
        assert_eq!(
            "IN_PROGRESS".parse::<TaskStatus>().unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!("DONE".parse::<TaskStatus>().unwrap(), TaskStatus::Done);
        assert_eq!(
            "CANCELLED".parse::<TaskStatus>().unwrap(),
            TaskStatus::Cancelled
        );
    }

    #[test]
    fn task_status_rejects_unknown_values() {
        assert!("SCHEDULED".parse::<TaskStatus>().is_err());
    }

    #[test]
    fn task_status_round_trips_through_as_str() {
        for status in [
            TaskStatus::Planned,
            TaskStatus::InProgress,
            TaskStatus::Done,
            TaskStatus::Cancelled,
        ] {
            assert_eq!(status.as_str().parse::<TaskStatus>().unwrap(), status);
        }
    }
}
