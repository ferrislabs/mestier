use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};

use crate::{paths::WorkflowPath, workflow::require_workflow};

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}",
    operation_id = "deleteWorkflow",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("workflow_id" = uuid::Uuid, Path, description = "Workflow identifier"),
    ),
    responses(
        (status = 204, description = "Workflow deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Workflow not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkflowPath {
        organization_id,
        workflow_id,
    }: WorkflowPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_workflow(&state, &identity, organization_id, workflow_id).await?;

    state
        .usecase
        .delete_workflow(organization_id, workflow_id)
        .await?;

    Ok(Response::NoContent)
}
