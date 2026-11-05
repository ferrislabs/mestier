use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateWorkflowCommand, WorkflowInitialState, WorkflowTags};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{paths::WorkflowsPath, require_manage_automation, response::WorkflowResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWorkflowRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub folder_id: Option<Uuid>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
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
        (status = 201, description = "Workflow created with no version yet; enabled unless the request says otherwise", body = inline(DataEnvelope<WorkflowResponse>)),
        (status = 400, description = "Validation failed, including more than 10 tags or a tag outside 1 to 30 characters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "The folder does not exist in this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: WorkflowsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateWorkflowRequest>,
) -> Result<Response<WorkflowResponse>, ApiError> {
    require_manage_automation(&state, &identity, path.organization_id).await?;

    let tags = WorkflowTags::try_from(payload.tags.unwrap_or_default())
        .map_err(|error| ApiError::Validation(error.to_string()))?;
    let workflow = state
        .usecase
        .create_workflow_in_state(
            CreateWorkflowCommand {
                org_id: path.organization_id,
                name: payload.name,
                description: payload.description,
            },
            WorkflowInitialState {
                enabled: payload.enabled.unwrap_or(true),
                folder_id: payload.folder_id,
                tags,
            },
        )
        .await?;

    Ok(Response::Created(WorkflowResponse::from(workflow)))
}
