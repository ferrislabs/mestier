use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    paths::RunPath,
    require_view_automation,
    response::{GraphDto, RunDetailResponse, RunResponse, RunStepResponse},
    run::find_run_in_org,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/runs/{run_id}",
    operation_id = "getRun",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("run_id" = uuid::Uuid, Path, description = "Run identifier"),
    ),
    responses(
        (status = 200, description = "Run with its steps — resolved input, output, error and attempts on each — and the graph its pinned version executed, null when that version is gone", body = inline(DataEnvelope<RunDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Run not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    RunPath {
        organization_id,
        run_id,
    }: RunPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<RunDetailResponse>, ApiError> {
    require_view_automation(&state, &identity, organization_id).await?;
    let run = find_run_in_org(&state, organization_id, run_id).await?;
    let steps = state
        .usecase
        .list_run_steps(organization_id, run_id)
        .await?;
    let graph = state
        .usecase
        .find_workflow_version_by_id(organization_id, run.workflow_version_id)
        .await?
        .map(|version| GraphDto::from(version.graph));

    Ok(Response::OK(RunDetailResponse {
        run: RunResponse::from(run),
        steps: steps.into_iter().map(RunStepResponse::from).collect(),
        graph,
    }))
}
