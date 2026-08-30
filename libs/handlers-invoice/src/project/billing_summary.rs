use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ProjectId;

use crate::{
    paths::ProjectBillingSummaryPath, require_view_invoices,
    response::ProjectBillingSummaryResponse,
};

/// "What was quoted, what has been billed, what remains" — `#[transactional]`
/// use case `project_billing_summary`, added by this issue since nothing
/// exposed the figure before (see the commit message).
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/billing-summary",
    operation_id = "getProjectBillingSummary",
    tag = super::super::TAG,
    params(
        ("project_id" = ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "The project's billing summary", body = inline(DataEnvelope<ProjectBillingSummaryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectBillingSummaryPath { project_id }: ProjectBillingSummaryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProjectBillingSummaryResponse>, ApiError> {
    let project = state.usecase.get_project(project_id).await?;
    require_view_invoices(&state, &identity, project.organization_id).await?;

    let summary = state.usecase.project_billing_summary(project_id).await?;

    Ok(Response::OK(summary.into()))
}
