use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    require_org_membership,
    task_label::{TaskLabelResponse, TaskLabelsPath},
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/task-labels",
    operation_id = "listTaskLabels",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "List of task labels", body = inline(DataEnvelope<Vec<TaskLabelResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TaskLabelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<TaskLabelResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let labels = state.usecase.list_task_labels(path.organization_id).await?;
    let items: Vec<TaskLabelResponse> = labels.into_iter().map(TaskLabelResponse::from).collect();

    Ok(Response::OK(items))
}
