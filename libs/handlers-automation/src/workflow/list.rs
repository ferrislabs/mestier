use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::WorkflowsPath, require_org_membership, response::WorkflowResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/workflows",
    operation_id = "listWorkflows",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Workflows for this organization", body = inline(DataEnvelope<Vec<WorkflowResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: WorkflowsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<WorkflowResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let workflows = state.usecase.list_workflows(path.organization_id).await?;
    let body: Vec<WorkflowResponse> = workflows.into_iter().map(WorkflowResponse::from).collect();

    Ok(Response::OK(body))
}
