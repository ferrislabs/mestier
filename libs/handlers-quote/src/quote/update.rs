use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CustomerContextId, CustomerId, QuoteId, QuoteStatus, UpdateQuoteCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::QuotePath,
    quote::create::{QuoteLineRequest, into_line_commands},
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

    let quote = state
        .usecase
        .update_quote(UpdateQuoteCommand {
            id: quote_id,
            title: payload.title,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            status: payload.status,
            lines: into_line_commands(payload.lines)?,
        })
        .await?;

    Ok(Response::OK(quote.into()))
}
