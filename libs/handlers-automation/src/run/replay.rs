use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::RunReplayPath, response::RunResponse, run::require_run};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReplayRunRequest {
    /// The graph-local connector id to restart from. Everything reachable
    /// from it re-executes on the next worker pass; everything upstream
    /// keeps its settled step and is read back, never re-executed.
    pub connector_id: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay",
    operation_id = "replayRun",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("run_id" = uuid::Uuid, Path, description = "Run identifier"),
    ),
    request_body = ReplayRunRequest,
    responses(
        (status = 200, description = "Run requeued, due immediately, starting at the named step", body = inline(DataEnvelope<RunResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Run not found"),
        (status = 409, description = "Unknown connector, or the run is still pending or running"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    RunReplayPath {
        organization_id,
        run_id,
    }: RunReplayPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ReplayRunRequest>,
) -> Result<Response<RunResponse>, ApiError> {
    require_run(&state, &identity, organization_id, run_id).await?;

    let replayed = state
        .usecase
        .replay_run_from_step(organization_id, run_id, payload.connector_id)
        .await?;

    Ok(Response::OK(RunResponse::from(replayed)))
}
