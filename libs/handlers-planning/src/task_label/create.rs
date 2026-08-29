use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateTaskLabelCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    require_org_membership,
    task_label::{TaskLabelResponse, TaskLabelsPath},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskLabelRequest {
    pub name: String,
    /// `#RRGGBB` hex value.
    pub color: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/task-labels",
    operation_id = "createTaskLabel",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateTaskLabelRequest,
    responses(
        (status = 201, description = "Task label created", body = inline(DataEnvelope<TaskLabelResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "A label with this name already exists in this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TaskLabelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateTaskLabelRequest>,
) -> Result<Response<TaskLabelResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let label = state
        .usecase
        .acting_as(user_id)
        .create_task_label(CreateTaskLabelCommand {
            actor,
            organization_id: path.organization_id,
            name: payload.name,
            color: payload.color,
        })
        .await?;

    Ok(Response::Created(label.into()))
}
