use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::TaskId;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    require_org_membership,
    response::{BulkAssignTasksResponse, TaskResponse},
    task::{TasksBulkAssignPath, update::AssigneeRefRequest},
};

/// Assigns the same complete set of `assignees` to every task in
/// `task_ids` — same replacement semantics as `PATCH /tasks/{id}`'s own
/// `assignees`, applied to many tasks in one request instead of one call
/// per task.
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkAssignTasksRequest {
    pub task_ids: Vec<TaskId>,
    pub assignees: Vec<AssigneeRefRequest>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/tasks/bulk-assign",
    operation_id = "bulkAssignTasks",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = BulkAssignTasksRequest,
    responses(
        (status = 200, description = "Every named task reassigned together, in one transaction", body = inline(DataEnvelope<BulkAssignTasksResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "A task, or a member, was not found — the whole batch is rolled back, none of the named tasks are reassigned"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TasksBulkAssignPath { organization_id }: TasksBulkAssignPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<BulkAssignTasksRequest>,
) -> Result<Response<BulkAssignTasksResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let assignees = payload.assignees.into_iter().map(Into::into).collect();
    let tasks = state
        .usecase
        .acting_as(actor)
        .bulk_assign_tasks(organization_id, payload.task_ids, assignees)
        .await?;

    // Same batched-fetch reasoning as `task::list`: one grouped query for
    // the whole response, never one per task.
    let task_ids: Vec<TaskId> = tasks.iter().map(|task| task.id).collect();
    let mut labels_by_task = state
        .usecase
        .list_task_labels_for_tasks(task_ids.clone())
        .await?;
    let mut equipment_by_task = state.usecase.list_equipment_for_tasks(task_ids).await?;

    let items: Vec<TaskResponse> = tasks
        .into_iter()
        .map(|task| {
            let labels = labels_by_task.remove(&task.id).unwrap_or_default();
            let equipment = equipment_by_task.remove(&task.id).unwrap_or_default();
            TaskResponse {
                labels: labels.into_iter().map(Into::into).collect(),
                equipment: equipment.into_iter().map(Into::into).collect(),
                ..task.into()
            }
        })
        .collect();

    Ok(Response::OK(BulkAssignTasksResponse { tasks: items }))
}
