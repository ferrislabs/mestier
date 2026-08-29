use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{LineAllocationShare, ReplaceSupplierInvoiceLineAllocationsCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::SupplierInvoiceLineAllocationsPath, require_project_membership,
    require_supplier_invoice_line_membership, response::SupplierInvoiceLineAllocationResponse,
};

/// One target project and its share of the line — the request body's own
/// element type, kept separate from [`LineAllocationShare`] so the wire
/// shape can evolve independently of the domain command's.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LineAllocationShareRequest {
    pub project_id: mestier_core::ProjectId,
    pub amount_cents: i32,
}

/// The complete, authoritative list of shares for this line — full-replace
/// semantics, same shape as `Task::assignments`' own `PUT`: what is not in
/// this list is deleted, what changed amount is replaced, what is
/// byte-identical to what is already stored is left untouched.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReplaceLineAllocationsRequest {
    pub allocations: Vec<LineAllocationShareRequest>,
}

#[utoipa::path(
    put,
    path = "/api/v1/supplier-invoice-lines/{supplier_invoice_line_id}/allocations",
    operation_id = "replaceSupplierInvoiceLineAllocations",
    tag = crate::TAG,
    params(
        ("supplier_invoice_line_id" = mestier_core::SupplierInvoiceLineId, Path, description = "Supplier invoice line identifier"),
    ),
    request_body = ReplaceLineAllocationsRequest,
    responses(
        (status = 200, description = "The line's allocations, as they now stand", body = inline(DataEnvelope<Vec<SupplierInvoiceLineAllocationResponse>>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Supplier invoice line, or one of its target projects, not found"),
        (status = 409, description = "Allocations would overflow the line's total, or mix currencies/signs"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SupplierInvoiceLineAllocationsPath {
        supplier_invoice_line_id,
    }: SupplierInvoiceLineAllocationsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ReplaceLineAllocationsRequest>,
) -> Result<Response<Vec<SupplierInvoiceLineAllocationResponse>>, ApiError> {
    let line =
        require_supplier_invoice_line_membership(&state, &identity, supplier_invoice_line_id)
            .await?;

    for share in &payload.allocations {
        require_project_membership(&state, &identity, share.project_id).await?;
    }

    let allocations = state
        .usecase
        .replace_supplier_invoice_line_allocations(ReplaceSupplierInvoiceLineAllocationsCommand {
            organization_id: line.organization_id,
            supplier_invoice_id: line.supplier_invoice_id,
            supplier_invoice_line_id,
            allocations: payload
                .allocations
                .into_iter()
                .map(|share| LineAllocationShare {
                    project_id: share.project_id,
                    amount_cents: share.amount_cents,
                })
                .collect(),
        })
        .await?;

    let items: Vec<SupplierInvoiceLineAllocationResponse> = allocations
        .into_iter()
        .map(SupplierInvoiceLineAllocationResponse::from)
        .collect();

    Ok(Response::OK(items))
}
