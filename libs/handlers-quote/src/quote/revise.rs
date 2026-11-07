use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::QuoteId;

use crate::{
    paths::QuoteRevisionPath, quote_error::QuoteFailure, require_quote_membership,
    response::QuoteResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/quotes/{quote_id}/revision",
    operation_id = "reviseQuote",
    tag = super::super::TAG,
    params(
        ("quote_id" = QuoteId, Path, description = "Identifier of the quote to revise"),
    ),
    responses(
        (status = 201, description = "The new draft revision", body = inline(DataEnvelope<QuoteResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
        (status = 409, description = "Only a sent or declined quote can be revised"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuoteRevisionPath { quote_id }: QuoteRevisionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<QuoteResponse>, QuoteFailure> {
    require_quote_membership(&state, &identity, quote_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let revision = state
        .usecase
        .acting_as(user_id)
        .revise_quote(quote_id, actor)
        .await??;

    Ok(Response::Created(revision.into()))
}
