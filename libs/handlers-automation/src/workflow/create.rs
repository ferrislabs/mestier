use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CreateWorkflowCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::WorkflowsPath, require_org_membership, response::WorkflowResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWorkflowRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/workflows",
    operation_id = "createWorkflow",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateWorkflowRequest,
    responses(
        (status = 201, description = "Workflow created, enabled, with no version yet", body = inline(DataEnvelope<WorkflowResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: WorkflowsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateWorkflowRequest>,
) -> Result<Response<WorkflowResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let workflow = state
        .usecase
        .create_workflow(CreateWorkflowCommand {
            org_id: path.organization_id,
            name: payload.name,
            description: payload.description,
        })
        .await?;

    Ok(Response::Created(WorkflowResponse::from(workflow)))
}
