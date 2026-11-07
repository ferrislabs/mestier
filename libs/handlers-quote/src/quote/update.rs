use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    CustomerContactId, CustomerContextId, CustomerId, DealId, MemberId, QuoteId, QuoteStatus,
    UpdateQuoteCommand,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::QuotePath,
    quote::create::{QuoteLineRequest, into_line_commands, into_terms},
    quote_error::QuoteFailure,
    require_quote_deal, require_quote_membership, require_quote_targets,
    response::QuoteResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateQuoteRequest {
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    #[serde(default)]
    pub deal_id: Option<DealId>,
    pub status: QuoteStatus,
    #[serde(default)]
    pub valid_until: Option<NaiveDate>,
    #[serde(default)]
    pub deposit_percent: Option<String>,
    #[serde(default)]
    pub owner_member_id: Option<MemberId>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub billing_contact_id: Option<CustomerContactId>,
    #[serde(default)]
    pub billing_address_id: Option<CustomerContextId>,
    pub lines: Vec<QuoteLineRequest>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/quotes/{quote_id}",
    operation_id = "updateQuote",
    tag = super::super::TAG,
    params(
        ("quote_id" = QuoteId, Path, description = "Quote identifier"),
    ),
    request_body = UpdateQuoteRequest,
    responses(
        (status = 200, description = "Quote updated", body = inline(DataEnvelope<QuoteResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
        (status = 409, description = "Quote locked or in conflict (E_QUOTE_LOCKED)"),
        (status = 422, description = "A rule refused the quote, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuotePath { quote_id }: QuotePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateQuoteRequest>,
) -> Result<Response<QuoteResponse>, QuoteFailure> {
    let current = require_quote_membership(&state, &identity, quote_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    require_quote_targets(
        &state,
        current.organization_id,
        payload.customer_id,
        payload.customer_context_id,
    )
    .await?;
    require_quote_deal(&state, current.organization_id, payload.deal_id).await?;

    let terms = into_terms(
        payload.valid_until,
        payload.deposit_percent,
        payload.owner_member_id,
        payload.notes,
        payload.billing_contact_id,
        payload.billing_address_id,
    )?;

    let quote = state
        .usecase
        .acting_as(user_id)
        .update_quote(
            UpdateQuoteCommand {
                id: quote_id,
                title: payload.title,
                customer_id: payload.customer_id,
                customer_context_id: payload.customer_context_id,
                deal_id: payload.deal_id,
                status: payload.status,
                terms,
                lines: into_line_commands(payload.lines)?,
            },
            actor,
        )
        .await??;

    Ok(Response::OK(quote.into()))
}
