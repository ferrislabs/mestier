use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{QuoteId, QuoteStatus, UpdateQuoteStatusCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::QuoteStatusPath, require_quote_membership, response::QuoteResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateQuoteStatusRequest {
    pub status: QuoteStatus,
}

#[utoipa::path(
    patch,
    path = "/api/v1/quotes/{quote_id}/status",
    operation_id = "updateQuoteStatus",
    tag = super::super::TAG,
    params(
        ("quote_id" = QuoteId, Path, description = "Quote identifier"),
    ),
    request_body = UpdateQuoteStatusRequest,
    responses(
        (status = 200, description = "Quote status updated", body = inline(DataEnvelope<QuoteResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuoteStatusPath { quote_id }: QuoteStatusPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateQuoteStatusRequest>,
) -> Result<Response<QuoteResponse>, ApiError> {
    require_quote_membership(&state, &identity, quote_id).await?;

    let quote = state
        .usecase
        .update_quote_status(UpdateQuoteStatusCommand {
            id: quote_id,
            status: payload.status,
        })
        .await?;

    Ok(Response::OK(quote.into()))
}
