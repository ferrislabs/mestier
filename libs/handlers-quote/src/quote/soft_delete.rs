use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{AppState, Response, resolve_actor};
use mestier_core::QuoteId;

use crate::{EmptyResponse, paths::QuotePath, quote_error::QuoteFailure, require_quote_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/quotes/{quote_id}",
    operation_id = "deleteQuote",
    tag = super::super::TAG,
    params(
        ("quote_id" = QuoteId, Path, description = "Quote identifier"),
    ),
    responses(
        (status = 204, description = "Quote soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
        (status = 409, description = "Only a draft can be deleted (E_QUOTE_LOCKED)"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuotePath { quote_id }: QuotePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, QuoteFailure> {
    require_quote_membership(&state, &identity, quote_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .soft_delete_quote(quote_id, actor)
        .await??;

    Ok(Response::NoContent)
}
