use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    doe::{ProjectDoePath, belongs_to, response::DoeResponse},
    require_org_membership,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/doe",
    operation_id = "getProjectDoe",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "The project's DOE", body = inline(DataEnvelope<DoeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "The project has no DOE"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectDoePath {
        organization_id,
        project_id,
    }: ProjectDoePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DoeResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let doe = belongs_to(
        state.usecase.get_project_doe(project_id).await?,
        organization_id,
    )?;

    Ok(Response::OK(doe.into()))
}
