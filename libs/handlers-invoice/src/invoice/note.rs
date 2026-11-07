use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{InvoiceId, NoteInvoiceCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    invoice_error::InvoiceFailure, paths::InvoiceNotePath, require_invoice_membership,
    response::InvoiceResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct NoteInvoiceRequest {
    pub note: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/invoices/{invoice_id}/note",
    operation_id = "noteInvoice",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    request_body = NoteInvoiceRequest,
    responses(
        (status = 200, description = "Internal note saved, a blank note clears it", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoiceNotePath { invoice_id }: InvoiceNotePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<NoteInvoiceRequest>,
) -> Result<Response<InvoiceResponse>, InvoiceFailure> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let invoice = state
        .usecase
        .acting_as(user_id)
        .note_invoice(NoteInvoiceCommand {
            id: invoice_id,
            actor,
            recorded_by: user_id,
            note: payload.note,
        })
        .await??;

    Ok(Response::OK(invoice.into()))
}
