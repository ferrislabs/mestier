use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{QuoteId, QuoteStatus, UpdateQuoteStatusCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::QuoteStatusPath, quote_error::QuoteFailure, require_quote_membership,
    response::QuoteResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateQuoteStatusRequest {
    pub status: QuoteStatus,
    #[serde(default)]
    pub responded_by: Option<String>,
    #[serde(default)]
    pub decline_reason: Option<String>,
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
        (status = 409, description = "State conflict (E_QUOTE_LOCKED, E_QUOTE_EXPIRED, E_QUOTE_NOT_SENT)"),
        (status = 422, description = "A rule refused the change, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuoteStatusPath { quote_id }: QuoteStatusPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateQuoteStatusRequest>,
) -> Result<Response<QuoteResponse>, QuoteFailure> {
    require_quote_membership(&state, &identity, quote_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let quote = state
        .usecase
        .acting_as(user_id)
        .update_quote_status(
            UpdateQuoteStatusCommand {
                id: quote_id,
                status: payload.status,
                responded_by: payload.responded_by,
                decline_reason: payload.decline_reason,
            },
            actor,
        )
        .await??;

    Ok(Response::OK(quote.into()))
}
