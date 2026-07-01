use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CustomerContextId, CustomerId, QuoteId, QuoteStatus, UpdateQuoteCommand};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    paths::QuotePath,
    quote::create::{QuoteLineRequest, into_legal_mention_template_ids, into_line_commands},
    require_quote_membership, require_quote_targets,
    response::QuoteResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateQuoteRequest {
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub status: QuoteStatus,
    pub lines: Vec<QuoteLineRequest>,
    #[serde(default)]
    pub legal_mention_template_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub deposit_basis: Option<String>,
    #[serde(default)]
    pub deposit_value: Option<String>,
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
        (status = 409, description = "Quote conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuotePath { quote_id }: QuotePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateQuoteRequest>,
) -> Result<Response<QuoteResponse>, ApiError> {
    let current = require_quote_membership(&state, &identity, quote_id).await?;
    require_quote_targets(
        &state,
        current.organization_id,
        payload.customer_id,
        payload.customer_context_id,
    )
    .await?;

    let deposit_value = match payload.deposit_value {
        Some(s) => Some(
            Decimal::from_str(&s)
                .map_err(|_| ApiError::Validation("deposit_value must be a decimal number".to_owned()))?,
        ),
        None => None,
    };

    let quote = state
        .usecase
        .update_quote(UpdateQuoteCommand {
            id: quote_id,
            title: payload.title,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            status: payload.status,
            deposit_basis: payload.deposit_basis,
            deposit_value,
            lines: into_line_commands(payload.lines)?,
            legal_mention_template_ids: into_legal_mention_template_ids(
                payload.legal_mention_template_ids,
            ),
        })
        .await?;

    Ok(Response::OK(quote.into()))
}
