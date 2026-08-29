use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{RejectSupplierInvoiceCommand, SupplierInvoiceId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::SupplierInvoiceRejectPath, require_supplier_invoice_membership,
    response::SupplierInvoiceResponse,
};

#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct RejectSupplierInvoiceRequest {
    pub notes: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/supplier-invoices/{supplier_invoice_id}/reject",
    operation_id = "rejectSupplierInvoice",
    tag = super::super::TAG,
    params(
        ("supplier_invoice_id" = SupplierInvoiceId, Path, description = "Supplier invoice identifier"),
    ),
    request_body = RejectSupplierInvoiceRequest,
    responses(
        (status = 200, description = "Supplier invoice rejected", body = inline(DataEnvelope<SupplierInvoiceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Supplier invoice not found"),
        (status = 409, description = "Already reviewed"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SupplierInvoiceRejectPath {
        supplier_invoice_id,
    }: SupplierInvoiceRejectPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<RejectSupplierInvoiceRequest>,
) -> Result<Response<SupplierInvoiceResponse>, ApiError> {
    require_supplier_invoice_membership(&state, &identity, supplier_invoice_id).await?;

    let invoice = state
        .usecase
        .reject_supplier_invoice(RejectSupplierInvoiceCommand {
            id: supplier_invoice_id,
            notes: payload.notes,
        })
        .await?;

    Ok(Response::OK(invoice.into()))
}
