use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    daf::{ProjectDafsPath, response::ProjectDafsResponse},
    require_org_membership,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/dafs",
    operation_id = "listDafs",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "The project's DAF dossier (prefilled and not saved when the project has none) and its DAF, by number", body = inline(DataEnvelope<ProjectDafsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectDafsPath {
        organization_id,
        project_id,
    }: ProjectDafsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ProjectDafsResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let (dossier, dafs) = state
        .usecase
        .project_dafs(organization_id, project_id)
        .await?;

    Ok(Response::OK(ProjectDafsResponse {
        dossier: dossier.into(),
        dafs: dafs.into_iter().map(Into::into).collect(),
    }))
}
