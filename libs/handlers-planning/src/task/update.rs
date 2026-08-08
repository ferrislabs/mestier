use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{
    AssigneeRef, EmployeeId, PatchTaskCommand, TaskId, TaskLabelId, TaskStatus, UserId,
};
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

use crate::{
    response::{PatchTaskResponse, TaskResponse},
    task::{TaskPath, require_task},
};

/// Distinguishes "the key is absent" (leave the field unchanged) from "the
/// key is present" (apply it, `null` included) — plain `Option<Option<T>>`
/// cannot make that distinction on its own because serde treats a missing
/// key and an explicit `null` the same way by default.
fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

#[derive(Debug, PartialEq, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssigneeRefRequest {
    Employee { employee_id: EmployeeId },
    Member { user_id: UserId },
}

impl From<AssigneeRefRequest> for AssigneeRef {
    fn from(value: AssigneeRefRequest) -> Self {
        match value {
            AssigneeRefRequest::Employee { employee_id } => AssigneeRef::Employee(employee_id),
            AssigneeRefRequest::Member { user_id } => AssigneeRef::Member(user_id),
        }
    }
}

/// Every field is optional and, when present, replaces the current value.
/// `parent_task_id`/`description`/`starts_at`/`ends_at` additionally
/// distinguish "absent" from "present but `null`" (see
/// [`deserialize_present`]) — `null` clears `parent_task_id` (the task
/// becomes a root) or `starts_at`/`ends_at` (the task reverts to inheriting
/// its parent's window). `title` cannot be cleared (the column is `NOT
/// NULL`), so a plain `Option<String>` is enough for it. `assignees` and
/// `label_ids` are each the complete replacement list, never a delta: an
/// absent key leaves the current set untouched, `[]` clears it entirely.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTaskRequest {
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<TaskId>, nullable)]
    pub parent_task_id: Option<Option<TaskId>>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<String>, nullable)]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<DateTime<Utc>>, nullable)]
    pub starts_at: Option<Option<DateTime<Utc>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<DateTime<Utc>>, nullable)]
    pub ends_at: Option<Option<DateTime<Utc>>>,
    #[serde(default)]
    pub all_day: Option<bool>,
    #[serde(default)]
    pub status: Option<TaskStatus>,
    #[serde(default)]
    pub blocks_availability: Option<bool>,
    #[serde(default)]
    pub assignees: Option<Vec<AssigneeRefRequest>>,
    #[serde(default)]
    pub label_ids: Option<Vec<TaskLabelId>>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/tasks/{task_id}",
    operation_id = "patchTask",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_id" = mestier_core::TaskId, Path, description = "Task identifier"),
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Task reparented, rescheduled and/or reassigned", body = inline(DataEnvelope<PatchTaskResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Task, employee or member not found"),
        (status = 409, description = "Task conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TaskPath {
        organization_id,
        task_id,
    }: TaskPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Response<PatchTaskResponse>, ApiError> {
    require_task(&state, &identity, organization_id, task_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let mut command = PatchTaskCommand::new(task_id);
    command.parent_task_id = payload.parent_task_id;
    command.title = payload.title;
    command.description = payload.description;
    command.starts_at = payload.starts_at;
    command.ends_at = payload.ends_at;
    command.all_day = payload.all_day;
    command.status = payload.status;
    command.blocks_availability = payload.blocks_availability;
    command.label_ids = payload.label_ids;
    command.assignees = payload
        .assignees
        .map(|assignees| assignees.into_iter().map(Into::into).collect());

    let (task, created_employees) = state.usecase.acting_as(actor).patch_task(command).await?;

    // Reflects the task's current labels regardless of whether this PATCH
    // touched them — a PATCH that never mentions `label_ids` still needs to
    // report the set it left untouched.
    let mut labels_by_task = state
        .usecase
        .list_task_labels_for_tasks(vec![task.id])
        .await?;
    let labels = labels_by_task.remove(&task.id).unwrap_or_default();

    Ok(Response::OK(PatchTaskResponse {
        task: TaskResponse {
            labels: labels.into_iter().map(Into::into).collect(),
            ..task.into()
        },
        created_employees: created_employees.into_iter().map(Into::into).collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: serde_json::Value) -> UpdateTaskRequest {
        serde_json::from_value(value).expect("payload must deserialize")
    }

    // ── absent vs. `null` on nullable fields ────────────────────────────

    #[test]
    fn absent_nullable_fields_leave_them_unset() {
        let request = parse(json!({}));

        assert_eq!(
            request.parent_task_id, None,
            "an absent `parent_task_id` key must not be mistaken for a clearing `null`"
        );
        assert_eq!(request.description, None);
        assert_eq!(request.starts_at, None);
        assert_eq!(request.ends_at, None);
    }

    #[test]
    fn null_parent_task_id_clears_it() {
        let request = parse(json!({ "parent_task_id": null }));

        assert_eq!(
            request.parent_task_id,
            Some(None),
            "an explicit `null` clears `parent_task_id`, turning the task back into a root"
        );
    }

    #[test]
    fn null_description_clears_it() {
        let request = parse(json!({ "description": null }));

        assert_eq!(request.description, Some(None));
    }

    #[test]
    fn null_starts_at_and_ends_at_clear_them_to_inherit_from_the_parent() {
        let request = parse(json!({ "starts_at": null, "ends_at": null }));

        assert_eq!(request.starts_at, Some(None));
        assert_eq!(request.ends_at, Some(None));
    }

    #[test]
    fn present_title_sets_the_new_value() {
        let request = parse(json!({ "title": "Nouveau titre" }));

        assert_eq!(request.title, Some("Nouveau titre".to_owned()));
    }

    #[test]
    fn absent_title_leaves_it_unset() {
        let request = parse(json!({}));

        assert_eq!(request.title, None);
    }

    #[test]
    fn present_blocks_availability_sets_the_new_value() {
        let request = parse(json!({ "blocks_availability": false }));

        assert_eq!(request.blocks_availability, Some(false));
    }

    // ── the tagged `assignees` union ────────────────────────────────────

    #[test]
    fn employee_kind_deserializes_to_the_employee_variant() {
        let employee_id: EmployeeId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let parsed: AssigneeRefRequest = serde_json::from_value(json!({
            "kind": "employee",
            "employee_id": employee_id.0.to_string(),
        }))
        .expect("an employee assignee must deserialize");

        match AssigneeRef::from(parsed) {
            AssigneeRef::Employee(id) => assert_eq!(id, employee_id),
            other => panic!("expected AssigneeRef::Employee, got {other:?}"),
        }
    }

    #[test]
    fn member_kind_deserializes_to_the_member_variant() {
        let user_id: UserId = "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let parsed: AssigneeRefRequest = serde_json::from_value(json!({
            "kind": "member",
            "user_id": user_id.0.to_string(),
        }))
        .expect("a member assignee must deserialize");

        match AssigneeRef::from(parsed) {
            AssigneeRef::Member(id) => assert_eq!(id, user_id),
            other => panic!("expected AssigneeRef::Member, got {other:?}"),
        }
    }

    #[test]
    fn unknown_assignee_kind_is_rejected_not_ignored() {
        let result: Result<AssigneeRefRequest, _> = serde_json::from_value(json!({
            "kind": "equipment",
            "equipment_id": "33333333-3333-3333-3333-333333333333",
        }));

        assert!(
            result.is_err(),
            "an unrecognized `kind` must fail to deserialize rather than being silently dropped"
        );
    }

    // ── empty vs. absent `assignees` ────────────────────────────────────

    #[test]
    fn empty_assignees_array_means_clear_every_assignment() {
        let request = parse(json!({ "assignees": [] }));

        assert_eq!(
            request.assignees,
            Some(Vec::new()),
            "`assignees: []` is a present, empty replacement list — it must drop every \
             assignment, not be mistaken for \"don't touch assignments\""
        );
    }

    #[test]
    fn absent_assignees_means_leave_assignments_untouched() {
        let request = parse(json!({}));

        assert_eq!(
            request.assignees, None,
            "an absent `assignees` key must leave current assignments untouched, which is a \
             different outcome than `assignees: []`"
        );
    }

    #[test]
    fn assignees_carries_both_kinds_together() {
        let employee_id: EmployeeId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let user_id: UserId = "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let request = parse(json!({
            "assignees": [
                { "kind": "employee", "employee_id": employee_id.0.to_string() },
                { "kind": "member", "user_id": user_id.0.to_string() },
            ]
        }));

        let assignees = request.assignees.expect("assignees must be present");
        assert_eq!(assignees.len(), 2);

        let resolved: Vec<AssigneeRef> = assignees.into_iter().map(Into::into).collect();
        assert!(matches!(resolved[0], AssigneeRef::Employee(id) if id == employee_id));
        assert!(matches!(resolved[1], AssigneeRef::Member(id) if id == user_id));
    }

    // ── empty vs. absent `label_ids` ────────────────────────────────────

    #[test]
    fn present_label_ids_sets_the_new_value() {
        let label_id: TaskLabelId = "77777777-7777-7777-7777-777777777777".parse().unwrap();
        let request = parse(json!({ "label_ids": [label_id.0.to_string()] }));

        assert_eq!(request.label_ids, Some(vec![label_id]));
    }

    #[test]
    fn absent_label_ids_means_leave_labels_untouched() {
        let request = parse(json!({}));

        assert_eq!(
            request.label_ids, None,
            "an absent `label_ids` key must leave the task's current labels untouched, which is \
             a different outcome than `label_ids: []`"
        );
    }

    #[test]
    fn empty_label_ids_array_means_clear_every_label() {
        let request = parse(json!({ "label_ids": [] }));

        assert_eq!(
            request.label_ids,
            Some(Vec::new()),
            "`label_ids: []` is a present, empty replacement list — it must drop every label, \
             not be mistaken for \"don't touch labels\""
        );
    }
}
