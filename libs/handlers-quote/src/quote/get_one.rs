use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::QuoteId;

use crate::{paths::QuotePath, require_quote_membership, response::QuoteResponse};

#[utoipa::path(
    get,
    path = "/api/v1/quotes/{quote_id}",
    operation_id = "getQuote",
    tag = super::super::TAG,
    params(
        ("quote_id" = QuoteId, Path, description = "Quote identifier"),
    ),
    responses(
        (status = 200, description = "Quote details", body = inline(DataEnvelope<QuoteResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuotePath { quote_id }: QuotePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<QuoteResponse>, ApiError> {
    let quote = require_quote_membership(&state, &identity, quote_id).await?;

    Ok(Response::OK(quote.into()))
}
