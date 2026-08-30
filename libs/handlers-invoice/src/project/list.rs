use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ProjectId;

use crate::{paths::ProjectInvoicesPath, require_view_invoices, response::InvoiceResponse};

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/invoices",
    operation_id = "listProjectInvoices",
    tag = super::super::TAG,
    params(
        ("project_id" = ProjectId, Path, description = "Project identifier"),
    ),
    responses(
        (status = 200, description = "Every non-deleted invoice against this project", body = inline(DataEnvelope<Vec<InvoiceResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectInvoicesPath { project_id }: ProjectInvoicesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<InvoiceResponse>>, ApiError> {
    let project = state.usecase.get_project(project_id).await?;
    require_view_invoices(&state, &identity, project.organization_id).await?;

    let invoices = state.usecase.list_invoices_by_project(project_id).await?;
    let items: Vec<InvoiceResponse> = invoices.into_iter().map(InvoiceResponse::from).collect();

    Ok(Response::OK(items))
}
