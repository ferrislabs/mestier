use auth::Identity;
use axum::{
    Extension,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use doc_render::DocxDoeRenderer;
use handlers::{ApiError, AppState};
use http::{
    StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};

use crate::doe::{
    DoeDocxPath,
    pdf::{content_disposition, export_filename, load_files, off_the_executor, render_error},
    require_doe,
};

const DOCX_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/does/{doe_id}/docx",
    operation_id = "exportDoeDocx",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_id" = mestier_core::DoeId, Path, description = "DOE identifier"),
    ),
    responses(
        (status = 200, description = "The DOE as an editable Word document, generated on demand. PDF plans and datasheets are replaced by a line naming them.", content_type = "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE not found"),
        (status = 409, description = "A file the DOE references is no longer in storage, or the DOE is too large to export"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DoeDocxPath {
        organization_id,
        doe_id,
    }: DoeDocxPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response, ApiError> {
    let doe = require_doe(&state, &identity, organization_id, doe_id).await?;
    let document = state.usecase.doe_document(doe_id).await?;
    let files = load_files(&state, &document).await?;

    let filename = export_filename(&doe.title, doe.revision.as_str(), "docx");
    let bytes = off_the_executor(move || {
        DocxDoeRenderer
            .render(&document, &files)
            .map_err(|error| render_error(&document, error))
    })
    .await?;

    Ok((
        StatusCode::OK,
        [
            (CONTENT_TYPE, DOCX_CONTENT_TYPE.to_owned()),
            (CONTENT_DISPOSITION, content_disposition(&filename)),
        ],
        Body::from(bytes),
    )
        .into_response())
}
