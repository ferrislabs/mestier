use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateInvoiceFromQuoteCommand, QuoteId, QuoteInvoiceMode};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    invoice_error::InvoiceFailure, paths::QuoteInvoicesPath, require_quote_membership,
    response::InvoiceResponse,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuoteInvoiceKind {
    Full,
    Deposit,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInvoiceFromQuoteRequest {
    pub mode: QuoteInvoiceKind,
    #[serde(default)]
    pub percent: Option<String>,
}

fn mode_of(payload: &CreateInvoiceFromQuoteRequest) -> Result<QuoteInvoiceMode, ApiError> {
    match payload.mode {
        QuoteInvoiceKind::Full => Ok(QuoteInvoiceMode::Full),
        QuoteInvoiceKind::Deposit => {
            let percent = payload
                .percent
                .as_deref()
                .ok_or_else(|| ApiError::Validation("a deposit needs a percent".to_owned()))?;
            Decimal::from_str(percent)
                .map(|percent| QuoteInvoiceMode::Deposit { percent })
                .map_err(|_| ApiError::Validation("percent must be decimal".to_owned()))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/quotes/{quote_id}/invoices",
    operation_id = "createInvoiceFromQuote",
    tag = super::super::TAG,
    params(
        ("quote_id" = QuoteId, Path, description = "Quote identifier"),
    ),
    request_body = CreateInvoiceFromQuoteRequest,
    responses(
        (status = 201, description = "Draft invoice created from the accepted quote, issued deposits deducted per rate", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
        (status = 409, description = "The quote is not accepted (E_CONFLICT) or already has an active invoice (E_QUOTE_ALREADY_INVOICED)"),
        (status = 422, description = "A rule refused the invoice: E_DEPOSIT_EXCEEDS_REMAINING or a percent out of range"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuoteInvoicesPath { quote_id }: QuoteInvoicesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateInvoiceFromQuoteRequest>,
) -> Result<Response<InvoiceResponse>, InvoiceFailure> {
    require_quote_membership(&state, &identity, quote_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;
    let mode = mode_of(&payload)?;

    let invoice = state
        .usecase
        .acting_as(user_id)
        .create_invoice_from_quote(CreateInvoiceFromQuoteCommand {
            quote_id,
            actor,
            mode,
        })
        .await??;

    Ok(Response::Created(invoice.into()))
}
