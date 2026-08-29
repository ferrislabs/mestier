use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ProjectId;

use crate::{
    paths::ProjectSupplierCostsPath, require_project_membership,
    response::ProjectSupplierCostsResponse,
};

/// The plain net sum of every allocation recorded against this project so
/// far — see [`ProjectSupplierCostsResponse`]'s own doc comment for why
/// this is not the same figure a profitability report states.
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/supplier-costs",
    operation_id = "getProjectSupplierCosts",
    tag = crate::TAG,
    params(
        ("project_id" = ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "Allocated supplier cost for this project", body = inline(DataEnvelope<ProjectSupplierCostsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectSupplierCostsPath { project_id }: ProjectSupplierCostsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProjectSupplierCostsResponse>, ApiError> {
    require_project_membership(&state, &identity, project_id).await?;

    let allocated_cents = state
        .usecase
        .allocated_supplier_cost_for_project(project_id)
        .await?;

    Ok(Response::OK(ProjectSupplierCostsResponse {
        project_id,
        allocated_cents,
    }))
}
