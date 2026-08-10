use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::WorkflowPath, response::WorkflowDetailResponse, workflow::require_workflow};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}",
    operation_id = "getWorkflow",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("workflow_id" = uuid::Uuid, Path, description = "Workflow identifier"),
    ),
    responses(
        (status = 200, description = "Workflow with its current version, if it has one", body = inline(DataEnvelope<WorkflowDetailResponse>)),
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
) -> Result<Response<WorkflowDetailResponse>, ApiError> {
    let workflow = require_workflow(&state, &identity, organization_id, workflow_id).await?;

    // `find_workflow_version` looks a version up by its *number*, and the
    // workflow only carries `current_version_id` (the version row's own
    // id) — so the current version is found by scanning the (small) set of
    // saved versions for the one `insert_version` last pointed at, rather
    // than adding a by-id lookup this is the crate's only caller of.
    let current_version = match workflow.current_version_id {
        Some(current_version_id) => state
            .usecase
            .list_workflow_versions(organization_id, workflow_id)
            .await?
            .into_iter()
            .find(|version| version.id == current_version_id),
        None => None,
    };

    Ok(Response::OK(WorkflowDetailResponse::new(
        workflow,
        current_version,
    )))
}
