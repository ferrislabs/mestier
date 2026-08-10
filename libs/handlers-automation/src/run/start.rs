use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use utoipa::ToSchema;

use crate::{paths::WorkflowRunsPath, workflow::require_workflow};

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartRunRequest {
    /// Read into every connector's `{{ trigger.* }}` expressions. Defaults
    /// to an empty object for a workflow that reads nothing from its
    /// trigger.
    #[serde(default)]
    pub trigger_payload: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct StartedRunResponse {
    pub run_id: uuid::Uuid,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/runs",
    operation_id = "startRun",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("workflow_id" = uuid::Uuid, Path, description = "Workflow identifier"),
    ),
    request_body = StartRunRequest,
    responses(
        (status = 201, description = "Run started manually, pending its first pass", body = inline(DataEnvelope<StartedRunResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Workflow not found"),
        (status = 409, description = "The workflow has no saved version to run"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkflowRunsPath {
        organization_id,
        workflow_id,
    }: WorkflowRunsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<StartRunRequest>,
) -> Result<Response<StartedRunResponse>, ApiError> {
    require_workflow(&state, &identity, organization_id, workflow_id).await?;
    let actor = handlers::resolve_user_id(&state, &identity).await?;

    let run_id = state
        .usecase
        .acting_as(actor)
        .start_run(
            organization_id,
            workflow_id,
            payload.trigger_payload.unwrap_or_else(|| json!({})),
        )
        .await?;

    Ok(Response::Created(StartedRunResponse { run_id }))
}
