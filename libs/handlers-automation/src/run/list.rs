use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::RunsPath, require_org_membership, response::RunResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/runs",
    operation_id = "listRuns",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Runs for this organization, most recent first", body = inline(DataEnvelope<Vec<RunResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: RunsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<RunResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let runs = state.usecase.list_runs(path.organization_id).await?;
    let body: Vec<RunResponse> = runs.into_iter().map(RunResponse::from).collect();

    Ok(Response::OK(body))
}
