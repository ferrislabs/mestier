use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{paths::OrganizationInvoicesPath, require_view_invoices, response::InvoiceResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/invoices",
    operation_id = "listInvoices",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of invoices", body = inline(DataEnvelope<Vec<InvoiceResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrganizationInvoicesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<InvoiceResponse>, ApiError> {
    require_view_invoices(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (invoices, total) = state
        .usecase
        .list_invoices(path.organization_id, per_page, offset)
        .await?;
    let items: Vec<InvoiceResponse> = invoices.into_iter().map(InvoiceResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
