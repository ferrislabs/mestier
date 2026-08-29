use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{SupplierInvoiceId, UpdateSupplierInvoiceNotesCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::SupplierInvoicePath, require_supplier_invoice_membership,
    response::SupplierInvoiceResponse,
};

/// Metadata only — `notes`, nothing else. Unlike an issued invoice's own
/// `PATCH`, this one has no status guard: a received document's own fields
/// (amounts, dates, lines) are somebody else's facts and are never
/// editable here at any status; only our own review note is.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSupplierInvoiceRequest {
    pub notes: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/supplier-invoices/{supplier_invoice_id}",
    operation_id = "updateSupplierInvoiceNotes",
    tag = super::super::TAG,
    params(
        ("supplier_invoice_id" = SupplierInvoiceId, Path, description = "Supplier invoice identifier"),
    ),
    request_body = UpdateSupplierInvoiceRequest,
    responses(
        (status = 200, description = "Notes updated", body = inline(DataEnvelope<SupplierInvoiceResponse>)),
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
    Json(payload): Json<UpdateSupplierInvoiceRequest>,
) -> Result<Response<SupplierInvoiceResponse>, ApiError> {
    require_supplier_invoice_membership(&state, &identity, supplier_invoice_id).await?;

    let invoice = state
        .usecase
        .update_supplier_invoice_notes(UpdateSupplierInvoiceNotesCommand {
            id: supplier_invoice_id,
            notes: payload.notes,
        })
        .await?;

    Ok(Response::OK(invoice.into()))
}
