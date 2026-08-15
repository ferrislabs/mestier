use chrono::{DateTime, Utc};
use mestier_core::{
    CustomerContextId, CustomerId, Equipment, EquipmentId, MemberId, OrganizationId, QuoteId, Task,
    TaskId, TaskStatus,
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
    /// The complete set of currently assigned employees — mirrors the
    /// `PATCH` contract, where `assignees` is always the full list.
    pub member_ids: Vec<MemberId>,
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
            member_ids: value
                .assignments
                .into_iter()
                .map(|assignment| assignment.member_id)
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
            assignments: vec![TaskAssignment {
                id: "66666666-6666-6666-6666-666666666666".parse().unwrap(),
                organization_id,
                task_id: id,
                member_id,
                created_at: now,
            }],
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
}
