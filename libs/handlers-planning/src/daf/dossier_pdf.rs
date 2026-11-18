use auth::Identity;
use axum::{Extension, extract::State, response::Response};
use handlers::{ApiError, AppState};

use crate::{
    daf::{
        DafDossierPdfPath,
        pdf::{dossier_filename, load_files, pdf_response, render_off_the_executor},
    },
    require_org_membership,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/dafs/pdf",
    operation_id = "exportDafDossierPdf",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "The dossier as a PDF, generated on demand: the summary then every DAF by number", content_type = "application/pdf"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
        (status = 409, description = "A file a DAF references is no longer in storage"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DafDossierPdfPath {
        organization_id,
        project_id,
    }: DafDossierPdfPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let document = state
        .usecase
        .daf_dossier_document(organization_id, project_id)
        .await?;
    let files = load_files(&state, &document).await?;

    let filename = dossier_filename(&document.title);
    let bytes = render_off_the_executor(document, files).await?;

    Ok(pdf_response(&filename, bytes))
}
