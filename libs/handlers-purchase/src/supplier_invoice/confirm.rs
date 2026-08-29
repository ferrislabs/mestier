use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{ConfirmSupplierInvoiceCommand, SupplierInvoiceId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::SupplierInvoiceConfirmPath, require_supplier_invoice_membership,
    response::SupplierInvoiceResponse,
};

#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct ConfirmSupplierInvoiceRequest {
    pub notes: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/supplier-invoices/{supplier_invoice_id}/confirm",
    operation_id = "confirmSupplierInvoice",
    tag = super::super::TAG,
    params(
        ("supplier_invoice_id" = SupplierInvoiceId, Path, description = "Supplier invoice identifier"),
    ),
    request_body = ConfirmSupplierInvoiceRequest,
    responses(
        (status = 200, description = "Supplier invoice confirmed", body = inline(DataEnvelope<SupplierInvoiceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Supplier invoice not found"),
        (status = 409, description = "Already reviewed"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SupplierInvoiceConfirmPath {
        supplier_invoice_id,
    }: SupplierInvoiceConfirmPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ConfirmSupplierInvoiceRequest>,
) -> Result<Response<SupplierInvoiceResponse>, ApiError> {
    require_supplier_invoice_membership(&state, &identity, supplier_invoice_id).await?;

    let invoice = state
        .usecase
        .confirm_supplier_invoice(ConfirmSupplierInvoiceCommand {
            id: supplier_invoice_id,
            notes: payload.notes,
        })
        .await?;

    Ok(Response::OK(invoice.into()))
}
