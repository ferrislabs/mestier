use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::SupplierInvoiceId;

use crate::{
    paths::SupplierInvoicePath,
    require_supplier_invoice_membership,
    response::{SupplierInvoiceLineAllocationResponse, SupplierInvoiceResponse},
};

/// The one route that reads a line's current allocations, so this is the
/// one place `SupplierInvoiceLineResponse::allocations` is ever populated
/// for real — see that field's own doc comment.
#[utoipa::path(
    get,
    path = "/api/v1/supplier-invoices/{supplier_invoice_id}",
    operation_id = "getSupplierInvoice",
    tag = super::super::TAG,
    params(
        ("supplier_invoice_id" = SupplierInvoiceId, Path, description = "Supplier invoice identifier"),
    ),
    responses(
        (status = 200, description = "Supplier invoice details", body = inline(DataEnvelope<SupplierInvoiceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Supplier invoice not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SupplierInvoicePath {
        supplier_invoice_id,
    }: SupplierInvoicePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SupplierInvoiceResponse>, ApiError> {
    let invoice =
        require_supplier_invoice_membership(&state, &identity, supplier_invoice_id).await?;
    let mut response = SupplierInvoiceResponse::from(invoice);

    for line in &mut response.lines {
        let allocations = state
            .usecase
            .list_supplier_invoice_line_allocations(line.id)
            .await?;
        line.allocations = allocations
            .into_iter()
            .map(SupplierInvoiceLineAllocationResponse::from)
            .collect();
    }

    Ok(Response::OK(response))
}
