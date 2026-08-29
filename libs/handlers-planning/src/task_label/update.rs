use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateTaskLabelCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::task_label::{TaskLabelPath, TaskLabelResponse, require_task_label};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTaskLabelRequest {
    pub name: String,
    /// `#RRGGBB` hex value.
    pub color: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/task-labels/{label_id}",
    operation_id = "updateTaskLabel",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("label_id" = mestier_core::TaskLabelId, Path, description = "Task label identifier"),
    ),
    request_body = UpdateTaskLabelRequest,
    responses(
        (status = 200, description = "Task label updated", body = inline(DataEnvelope<TaskLabelResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Task label not found"),
        (status = 409, description = "A label with this name already exists in this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TaskLabelPath {
        organization_id,
        label_id,
    }: TaskLabelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateTaskLabelRequest>,
) -> Result<Response<TaskLabelResponse>, ApiError> {
    require_task_label(&state, &identity, organization_id, label_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let label = state
        .usecase
        .acting_as(user_id)
        .update_task_label(UpdateTaskLabelCommand {
            actor,
            id: label_id,
            name: payload.name,
            color: payload.color,
        })
        .await?;

    Ok(Response::OK(label.into()))
}
