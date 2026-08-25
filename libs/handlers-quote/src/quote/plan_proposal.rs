use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    paths::QuotePlanProposalPath, require_quote_membership, response::QuotePlanProposalResponse,
};

/// A read that proposes, never a write: one suggested task per quote line,
/// for a human to review before `POST .../plan` confirms anything. A quote
/// line is a commercial unit and a task is a scheduling unit — a 40 hour
/// line of terrassement is not one task, so this never invents that
/// mapping either, only a title and (when it can be read off the line, not
/// guessed) a duration.
#[utoipa::path(
    get,
    path = "/api/v1/quotes/{quote_id}/plan-proposal",
    operation_id = "getQuotePlanProposal",
    tag = super::super::TAG,
    params(
        ("quote_id" = mestier_core::QuoteId, Path, description = "Quote identifier"),
    ),
    responses(
        (status = 200, description = "One suggested task per quote line", body = inline(DataEnvelope<QuotePlanProposalResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
        (status = 409, description = "The quote is not accepted"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuotePlanProposalPath { quote_id }: QuotePlanProposalPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<QuotePlanProposalResponse>, ApiError> {
    require_quote_membership(&state, &identity, quote_id).await?;

    let (quote, proposal) = state.usecase.get_quote_plan_proposal(quote_id).await?;

    Ok(Response::OK(QuotePlanProposalResponse {
        quote: quote.into(),
        tasks: proposal.into_iter().map(Into::into).collect(),
    }))
}
