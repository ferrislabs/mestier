use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{InvoiceId, IssueCreditNoteCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    invoice::create::{InvoiceLineRequest, into_line_commands},
    paths::InvoiceCreditNotesPath,
    require_invoice_membership,
    response::InvoiceResponse,
};

/// `source_invoice_id` is never part of this body: it comes from the path,
/// same rule as the command it builds (`IssueCreditNoteCommand`'s own doc:
/// a credit note's counterparty cannot differ from the invoice it
/// corrects).
#[derive(Debug, Deserialize, ToSchema)]
pub struct IssueCreditNoteRequest {
    pub lines: Vec<InvoiceLineRequest>,
    pub notes: Option<String>,
    #[serde(default)]
    pub allow_exceeding_invoice_total: bool,
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
        (status = 409, description = "The source invoice cannot be credited, or crediting it would exceed its net total"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoiceCreditNotesPath { invoice_id }: InvoiceCreditNotesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<IssueCreditNoteRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let credit_note = state
        .usecase
        .acting_as(actor)
        .issue_credit_note(IssueCreditNoteCommand {
            source_invoice_id: invoice_id,
            lines: into_line_commands(payload.lines)?,
            notes: payload.notes,
            allow_exceeding_invoice_total: payload.allow_exceeding_invoice_total,
        })
        .await?;

    Ok(Response::Created(credit_note.into()))
}
