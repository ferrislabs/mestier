use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateDafDossierCommand;

use crate::{
    daf::{DafDossierPath, request::UpdateDafDossierRequest, response::DafDossierResponse},
    require_org_membership,
};

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/daf-dossier",
    operation_id = "saveDafDossier",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    request_body = UpdateDafDossierRequest,
    responses(
        (status = 200, description = "The dossier, saved", body = inline(DataEnvelope<DafDossierResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
        (status = 409, description = "A blank title or issuer"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DafDossierPath {
        organization_id,
        project_id,
    }: DafDossierPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateDafDossierRequest>,
) -> Result<Response<DafDossierResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let dossier = state
        .usecase
        .save_daf_dossier(UpdateDafDossierCommand {
            actor,
            organization_id,
            project_id,
            title: payload.title,
            issuer: payload.issuer,
            project_owner: payload.project_owner,
            project_manager: payload.project_manager,
        })
        .await?;

    Ok(Response::OK(dossier.into()))
}
