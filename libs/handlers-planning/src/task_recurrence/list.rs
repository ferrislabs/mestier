use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    require_org_membership, response::TaskRecurrenceResponse, task_recurrence::TaskRecurrencesPath,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/task-recurrences",
    operation_id = "listTaskRecurrences",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Every non-deleted recurrence of this organization", body = inline(DataEnvelope<Vec<TaskRecurrenceResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TaskRecurrencesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<TaskRecurrenceResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let recurrences = state
        .usecase
        .list_task_recurrences(path.organization_id)
        .await?;

    let items: Vec<TaskRecurrenceResponse> = recurrences.into_iter().map(Into::into).collect();

    Ok(Response::OK(items))
}
