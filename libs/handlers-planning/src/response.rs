use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use mestier_core::{
    CustomerContextId, CustomerId, Equipment, EquipmentId, MemberId, OrganizationId, Project,
    ProjectId, QuoteId, Task, TaskAssignmentId, TaskId, TaskRecurrence, TaskRecurrenceId,
    TaskStatus, weekday_to_iso,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::task_label::TaskLabelResponse;

/// The equipment attached to a task, embedded in `TaskResponse.equipment` —
/// mirrors `TaskLabelResponse`'s own reasoning: the full object, not just an
/// id, so the front can show name and hourly rate without a second call.
/// Defined here rather than reused from `handlers-reference` (which owns
/// equipment's own CRUD) since this crate does not depend on that one.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TaskEquipmentResponse {
    pub id: EquipmentId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub hourly_rate_cents: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Equipment> for TaskEquipmentResponse {
    fn from(value: Equipment) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            hourly_rate_cents: value.hourly_rate_cents,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// One assignee, with the id of the assignment itself — see
/// `TaskResponse::assignments`'s own doc comment for why this exists
/// alongside `member_ids`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, ToSchema)]
pub struct TaskAssignmentSummary {
    pub id: TaskAssignmentId,
    pub member_id: MemberId,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TaskResponse {
    pub id: TaskId,
    pub organization_id: OrganizationId,
    pub parent_task_id: Option<TaskId>,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub status: TaskStatus,
    pub blocks_availability: bool,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
    /// The project this task is costed against, if any. Independent of
    /// `parent_task_id`: a subtask may name a project its parent does not.
    pub project_id: Option<ProjectId>,
    /// The series this task was materialized from, when it still follows
    /// one. `None` for a task that never belonged to a series, or for one
    /// that did and was detached by an edit — see `PatchTaskResponse::detached`
    /// for the moment that transition is reported.
    pub recurrence_id: Option<TaskRecurrenceId>,
    /// What the task costs beyond somebody's time. `0` with no label when there
    /// is nothing to declare.
    pub expenses_cents: i32,
    pub expenses_label: Option<String>,
    /// The complete set of currently assigned employees — mirrors the
    /// `PATCH` contract, where `assignees` is always the full list.
    pub member_ids: Vec<MemberId>,
    /// Same set as `member_ids`, carrying each assignment's own id
    /// alongside its member — what `assignment_report`'s `PATCH
    /// .../resolution` and the task sheet's pending-report panel need to
    /// tell one assignee's correction from another's on a multi-assignee
    /// task. Kept as a separate field rather than folded into `member_ids`
    /// to avoid a wire-format break for the existing readers of that field.
    pub assignments: Vec<TaskAssignmentSummary>,
    /// The number of direct children — only `GET /tasks` computes this (one
    /// grouped query per page, see `TaskRepository::count_children`); every
    /// other endpoint leaves it `None` rather than pay for an extra query
    /// or report a stale/wrong `0`.
    pub child_count: Option<i64>,
    /// The complete set of labels currently attached to this task — the
    /// full object (`id`, `name`, `color`), not just an id, so the front
    /// can paint a colored chip without a second call. Always `[]`, never
    /// `null`, for a task with none.
    ///
    /// `From<Task>` below defaults this to empty: `Task` itself does not
    /// carry labels (see the planning module design doc — `task` and
    /// `task_label` are deliberately separate aggregates), so every
    /// handler that wants the real set fetches it separately, batched
    /// across the whole response in one call
    /// (`MestierUseCase::list_task_labels_for_tasks`) — see
    /// `task::{create,get_one,list,update}` for how each of the four
    /// surfaces that return this type populates it.
    pub labels: Vec<TaskLabelResponse>,
    /// The complete set of equipment currently attached to this task — same
    /// reasoning as `labels`: always `[]`, never `null`, for a task with
    /// none, and populated the same way (`Task` does not carry equipment
    /// either, see `TaskEquipmentResponse`'s own doc comment).
    pub equipment: Vec<TaskEquipmentResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Task> for TaskResponse {
    fn from(value: Task) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            parent_task_id: value.parent_task_id,
            title: value.title,
            description: value.description,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            all_day: value.all_day,
            status: value.status,
            blocks_availability: value.blocks_availability,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            quote_id: value.quote_id,
            project_id: value.project_id,
            recurrence_id: value.recurrence_id,
            expenses_cents: value.expenses_cents,
            expenses_label: value.expenses_label,
            member_ids: value
                .assignments
                .iter()
                .map(|assignment| assignment.member_id)
                .collect(),
            assignments: value
                .assignments
                .into_iter()
                .map(|assignment| TaskAssignmentSummary {
                    id: assignment.id,
                    member_id: assignment.member_id,
                })
                .collect(),
            child_count: None,
            // A task freshly loaded from `Task` alone (never labeled by
            // `Task` itself) has no labels until a caller fetches and
            // overlays the real set — see `labels`'s own doc comment. A
            // task fresh out of `POST /tasks` cannot have any yet regardless
            // (a label link is only ever created by a later `PATCH`), so
            // `create.rs` leaves this default as is, no extra query.
            labels: Vec::new(),
            // Same reasoning as `labels` just above: no caller populates
            // equipment for a bare `Task`, and a task fresh out of `POST
            // /tasks` cannot have any yet regardless.
            equipment: Vec::new(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// Response body for the transactional `PATCH`: the task as it stands after
/// reparenting, rescheduling or reassignment.
///
/// It used to carry `created_employees` as well — every HR record provisioned
/// on the fly because a bare member could not be assigned. Every member is
/// assignable now, so nothing is provisioned and there is nothing to report
/// back.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PatchTaskResponse {
    pub task: TaskResponse,
    /// Whether this `PATCH` detached the task from the series it belonged
    /// to before the call — the screen's only way to explain why editing
    /// this occurrence did not also change next Tuesday's. `false` for a
    /// task that was never part of a series, or one already detached before
    /// this call.
    pub detached: bool,
}

/// Response body for the bulk-assignment endpoint: every task named in the
/// request, as it stands after the same `assignees` set was applied to all
/// of them in one transaction.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct BulkAssignTasksResponse {
    pub tasks: Vec<TaskResponse>,
}

/// The rule, on the wire: `frequency` discriminates, and only the field the
/// chosen frequency actually needs is present — mirrors
/// `mestier_core::RecurrenceRule`'s own shape, and the same reasoning: an
/// RRULE string would let a `WEEKLY` rule show up with no weekday, this
/// cannot.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
#[serde(tag = "frequency", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecurrenceRuleResponse {
    Daily,
    /// ISO weekday numbers, 1 (Monday) through 7 (Sunday).
    Weekly {
        weekdays: Vec<i16>,
    },
    /// 1 through 31, clamped to the month's own last day when it is shorter
    /// — see `mestier_core`'s `expand_occurrences`.
    Monthly {
        day_of_month: u8,
    },
}

impl From<mestier_core::RecurrenceRule> for RecurrenceRuleResponse {
    fn from(value: mestier_core::RecurrenceRule) -> Self {
        match value {
            mestier_core::RecurrenceRule::Daily => Self::Daily,
            mestier_core::RecurrenceRule::Weekly { weekdays } => Self::Weekly {
                weekdays: weekdays.into_iter().map(weekday_to_iso).collect(),
            },
            mestier_core::RecurrenceRule::Monthly { day_of_month } => {
                Self::Monthly { day_of_month }
            }
        }
    }
}

/// A recurrence, and the template every occurrence it materializes copies —
/// see `mestier_core::TaskRecurrence`'s own doc. `RecurrenceFrequency` is
/// re-derivable from `rule` but not sent separately: `rule`'s own `frequency`
/// tag already carries it, and a second field would just be one more way for
/// the two to disagree.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TaskRecurrenceResponse {
    pub id: TaskRecurrenceId,
    pub organization_id: OrganizationId,
    #[serde(flatten)]
    pub rule: RecurrenceRuleResponse,
    pub starts_on: NaiveDate,
    pub ends_on: Option<NaiveDate>,
    /// Every occurrence up to and including this date already exists as a
    /// `tasks` row.
    pub horizon_filled_to: NaiveDate,
    /// IANA zone name (e.g. `Europe/Paris`) `start_time` is interpreted in.
    pub timezone: String,
    pub start_time: NaiveTime,
    pub duration_minutes: i32,
    pub all_day: bool,
    pub title: String,
    pub description: Option<String>,
    pub blocks_availability: bool,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub project_id: Option<ProjectId>,
    pub assignee_member_ids: Vec<MemberId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TaskRecurrence> for TaskRecurrenceResponse {
    fn from(value: TaskRecurrence) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            rule: value.rule.into(),
            starts_on: value.starts_on,
            ends_on: value.ends_on,
            horizon_filled_to: value.horizon_filled_to,
            timezone: value.timezone.to_string(),
            start_time: value.start_time,
            duration_minutes: value.duration_minutes,
            all_day: value.all_day,
            title: value.title,
            description: value.description,
            blocks_availability: value.blocks_availability,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            project_id: value.project_id,
            assignee_member_ids: value.assignee_member_ids,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[cfg(test)]
mod recurrence_response_tests {
    use super::*;
    use chrono::NaiveDate;

    fn recurrence() -> TaskRecurrence {
        let now = Utc::now();
        TaskRecurrence {
            id: "77777777-7777-7777-7777-777777777777".parse().unwrap(),
            organization_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
            rule: mestier_core::RecurrenceRule::Weekly {
                weekdays: vec![chrono::Weekday::Tue],
            },
            starts_on: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            ends_on: None,
            horizon_filled_to: NaiveDate::from_ymd_opt(2026, 9, 30).unwrap(),
            timezone: chrono_tz::Europe::Paris,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            duration_minutes: 60,
            all_day: false,
            title: "Réunion hebdo".to_owned(),
            description: None,
            blocks_availability: true,
            customer_id: None,
            customer_context_id: None,
            project_id: None,
            assignee_member_ids: Vec::new(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn a_weekly_rule_reports_its_iso_weekdays() {
        let response: TaskRecurrenceResponse = recurrence().into();

        assert_eq!(
            response.rule,
            RecurrenceRuleResponse::Weekly { weekdays: vec![2] }
        );
    }

    #[test]
    fn the_rule_is_flattened_onto_the_top_level_object() {
        let response: TaskRecurrenceResponse = recurrence().into();

        let value = serde_json::to_value(&response).unwrap();

        assert_eq!(value["frequency"], "WEEKLY");
        assert_eq!(value["weekdays"], serde_json::json!([2]));
        assert!(
            value.get("rule").is_none(),
            "the rule is flattened, not nested under its own key"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mestier_core::TaskAssignment;

    // `uuid` is not a direct dependency of `handlers-planning` (a
    // `Cargo.toml` this workstream does not own), so fixture ids are parsed
    // from literal strings via `FromStr` rather than generated.
    fn task() -> Task {
        let now = Utc::now();
        let id: TaskId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let organization_id: OrganizationId =
            "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let member_id: mestier_core::MemberId =
            "33333333-3333-3333-3333-333333333333".parse().unwrap();
        Task {
            id,
            organization_id,
            parent_task_id: None,
            title: "Toiture".to_owned(),
            description: None,
            starts_at: Some(now),
            ends_at: Some(now + chrono::Duration::hours(2)),
            all_day: false,
            status: TaskStatus::Planned,
            blocks_availability: true,
            customer_id: Some("44444444-4444-4444-4444-444444444444".parse().unwrap()),
            customer_context_id: Some("55555555-5555-5555-5555-555555555555".parse().unwrap()),
            quote_id: None,
            project_id: None,
            expenses_cents: 0,
            expenses_label: None,
            assignments: vec![TaskAssignment {
                id: "66666666-6666-6666-6666-666666666666".parse().unwrap(),
                organization_id,
                task_id: id,
                member_id,
                created_at: now,
            }],
            recurrence_id: None,
            occurrence_date: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn task_response_flattens_assignments_to_member_ids() {
        let source = task();
        let expected_member_id = source.assignments[0].member_id;

        let response: TaskResponse = source.into();

        assert_eq!(response.member_ids, vec![expected_member_id]);
    }

    /// The acceptance criterion `TaskResponse::assignments`'s own doc
    /// comment states: unlike `member_ids`, each entry also carries its own
    /// assignment id — what the task sheet needs to tell one assignee's
    /// pending report from another's.
    #[test]
    fn task_response_carries_each_assignment_id_alongside_its_member() {
        let source = task();
        let expected_id = source.assignments[0].id;
        let expected_member_id = source.assignments[0].member_id;

        let response: TaskResponse = source.into();

        assert_eq!(response.assignments.len(), 1);
        assert_eq!(response.assignments[0].id, expected_id);
        assert_eq!(response.assignments[0].member_id, expected_member_id);
    }

    #[test]
    fn task_response_leaves_child_count_unpopulated_by_default() {
        let response: TaskResponse = task().into();

        assert_eq!(
            response.child_count, None,
            "only the GET /tasks list handler populates child_count"
        );
    }

    #[test]
    fn task_response_serializes_no_labels_as_an_empty_array_not_null() {
        let response: TaskResponse = task().into();
        assert_eq!(response.labels, Vec::new());

        let value = serde_json::to_value(&response).unwrap();

        assert_eq!(
            value["labels"],
            serde_json::json!([]),
            "a task with no labels must serialize `labels` as `[]`, not `null`"
        );
    }

    #[test]
    fn task_response_serializes_no_equipment_as_an_empty_array_not_null() {
        let response: TaskResponse = task().into();
        assert_eq!(response.equipment, Vec::new());

        let value = serde_json::to_value(&response).unwrap();

        assert_eq!(
            value["equipment"],
            serde_json::json!([]),
            "a task with no equipment must serialize `equipment` as `[]`, not `null`"
        );
    }

    #[test]
    fn task_response_carries_both_equipment_of_a_two_equipment_task() {
        let now = Utc::now();
        let equipment_a = TaskEquipmentResponse {
            id: "99999999-9999-9999-9999-999999999999".parse().unwrap(),
            organization_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
            name: "Camion".to_owned(),
            hourly_rate_cents: 1500,
            created_at: now,
            updated_at: now,
        };
        let equipment_b = TaskEquipmentResponse {
            id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".parse().unwrap(),
            organization_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
            name: "Tondeuse".to_owned(),
            hourly_rate_cents: 800,
            created_at: now,
            updated_at: now,
        };

        let response = TaskResponse {
            equipment: vec![equipment_a.clone(), equipment_b.clone()],
            ..task().into()
        };

        assert_eq!(response.equipment, vec![equipment_a, equipment_b]);

        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(value["equipment"][0]["name"], "Camion");
        assert_eq!(value["equipment"][1]["name"], "Tondeuse");
    }

    #[test]
    fn task_response_carries_both_labels_of_a_two_label_task() {
        let now = Utc::now();
        let label_a = TaskLabelResponse {
            id: "77777777-7777-7777-7777-777777777777".parse().unwrap(),
            organization_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
            name: "Urgent".to_owned(),
            color: "#DC2626".to_owned(),
            created_at: now,
            updated_at: now,
        };
        let label_b = TaskLabelResponse {
            id: "88888888-8888-8888-8888-888888888888".parse().unwrap(),
            organization_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
            name: "Réunion".to_owned(),
            color: "#2563EB".to_owned(),
            created_at: now,
            updated_at: now,
        };

        let response = TaskResponse {
            labels: vec![label_a.clone(), label_b.clone()],
            ..task().into()
        };

        assert_eq!(response.labels, vec![label_a, label_b]);

        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(value["labels"][0]["name"], "Urgent");
        assert_eq!(value["labels"][1]["name"], "Réunion");
    }

    fn project(customer: bool) -> Project {
        let now = Utc::now();

        Project {
            id: "77777777-7777-7777-7777-777777777777".parse().unwrap(),
            organization_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
            name: "Réunion hebdo".to_owned(),
            customer_id: customer.then(|| "44444444-4444-4444-4444-444444444444".parse().unwrap()),
            customer_context_id: None,
            quote_id: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn a_project_with_no_customer_is_reported_as_internal() {
        let response: ProjectResponse = project(false).into();

        assert!(
            response.is_internal,
            "the front should not have to know that internal means customer-less"
        );
        assert_eq!(response.customer_id, None);
    }

    #[test]
    fn a_project_with_a_customer_is_not_internal() {
        let response: ProjectResponse = project(true).into();

        assert!(!response.is_internal);
    }

    #[test]
    fn task_response_carries_the_project_and_the_expenses() {
        let mut source = task();
        source.project_id = Some("77777777-7777-7777-7777-777777777777".parse().unwrap());
        source.expenses_cents = 4_500;
        source.expenses_label = Some("Déplacement Clermont".to_owned());

        let response: TaskResponse = source.into();

        assert_eq!(
            response.project_id,
            Some("77777777-7777-7777-7777-777777777777".parse().unwrap())
        );
        assert_eq!(response.expenses_cents, 4_500);
        assert_eq!(
            response.expenses_label.as_deref(),
            Some("Déplacement Clermont")
        );
    }
}

/// A project as the API returns it.
///
/// `customer_id` is `null` for internal work, and that is a state rather than
/// missing data: it is what lets a recurring meeting be a cost centre.
/// `archived_at` carries the whole lifecycle — there is no separate status.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id: ProjectId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
    pub archived_at: Option<DateTime<Utc>>,
    /// Derived rather than stored, so the front does not have to know that
    /// "internal" means "no customer".
    pub is_internal: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Project> for ProjectResponse {
    fn from(value: Project) -> Self {
        Self {
            is_internal: value.is_internal(),
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            quote_id: value.quote_id,
            archived_at: value.archived_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
