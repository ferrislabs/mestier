use auth::Identity;
use axum::{Extension, Json, extract::State};
use std::str::FromStr;

use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
    CreditLineRequest, CreditRequest, InvoiceId, InvoiceLineId, IssueCreditNoteCommand,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    invoice_error::InvoiceFailure, paths::InvoiceCreditNotesPath, require_invoice_membership,
    response::InvoiceResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreditLineBody {
    pub source_line_id: InvoiceLineId,
    pub quantity: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct IssueCreditNoteRequest {
    pub reason: String,
    #[serde(default)]
    pub full: bool,
    #[serde(default)]
    pub lines: Vec<CreditLineBody>,
    pub notes: Option<String>,
}

fn credit_request(payload: &IssueCreditNoteRequest) -> Result<CreditRequest, ApiError> {
    if payload.full {
        return Ok(CreditRequest::Full);
    }

    payload
        .lines
        .iter()
        .map(|line| {
            Decimal::from_str(&line.quantity)
                .map(|quantity| CreditLineRequest {
                    source_line_id: line.source_line_id,
                    quantity,
                })
                .map_err(|_| {
                    ApiError::Validation("credit line quantity must be decimal".to_owned())
                })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(CreditRequest::Lines)
}

#[utoipa::path(
    post,
    path = "/api/v1/invoices/{invoice_id}/credit-notes",
    operation_id = "issueCreditNote",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Source invoice identifier"),
    ),
    request_body = IssueCreditNoteRequest,
    responses(
        (status = 201, description = "Credit note issued", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Source invoice not found"),
        (status = 409, description = "The source invoice is not issued (E_CONFLICT)"),
        (status = 422, description = "A rule refused the credit note: reason required, E_CREDIT_EXCEEDS_REMAINING, E_INVOICE_DATE_RANGE"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoiceCreditNotesPath { invoice_id }: InvoiceCreditNotesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<IssueCreditNoteRequest>,
) -> Result<Response<InvoiceResponse>, InvoiceFailure> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;
    let request = credit_request(&payload)?;

    let credit_note = state
        .usecase
        .acting_as(user_id)
        .issue_credit_note(IssueCreditNoteCommand {
            source_invoice_id: invoice_id,
            actor,
            reason: payload.reason,
            request,
            notes: payload.notes,
        })
        .await??;

    Ok(Response::Created(credit_note.into()))
}

/// A document has few corrections, so this reads the full list rather than
/// paginating it — same call `list_handler` mirrors from
/// `payment::list::handler`.
#[utoipa::path(
    get,
    path = "/api/v1/invoices/{invoice_id}/credit-notes",
    operation_id = "listInvoiceCreditNotes",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Source invoice identifier"),
    ),
    responses(
        (status = 200, description = "Every credit note issued against this invoice", body = inline(DataEnvelope<Vec<InvoiceResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Source invoice not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_handler(
    InvoiceCreditNotesPath { invoice_id }: InvoiceCreditNotesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<InvoiceResponse>>, ApiError> {
    let invoice = state.usecase.get_invoice(invoice_id).await?;
    crate::require_view_invoices(&state, &identity, invoice.organization_id).await?;

    let credit_notes = state.usecase.get_invoice_credit_notes(invoice_id).await?;
    let items: Vec<InvoiceResponse> = credit_notes
        .into_iter()
        .map(InvoiceResponse::from)
        .collect();

    Ok(Response::OK(items))
}
