use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{
    paths::OrganizationSupplierInvoicesPath, require_org_membership,
    response::SupplierInvoiceResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/supplier-invoices",
    operation_id = "listSupplierInvoices",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of supplier invoices", body = inline(DataEnvelope<Vec<SupplierInvoiceResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrganizationSupplierInvoicesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<SupplierInvoiceResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (invoices, total) = state
        .usecase
        .list_supplier_invoices(path.organization_id, per_page, offset)
        .await?;
    let items: Vec<SupplierInvoiceResponse> = invoices
        .into_iter()
        .map(SupplierInvoiceResponse::from)
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
