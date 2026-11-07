use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{AppState, Response};
use mestier_core::DealId;

use crate::{
    DEALS_TAG, EmptyResponse, deal_error::DealFailure, paths::DealPath, require_org_membership,
};

#[utoipa::path(
    delete,
    path = "/api/v1/deals/{deal_id}",
    operation_id = "deleteDeal",
    tag = DEALS_TAG,
    params(
        ("deal_id" = DealId, Path, description = "Deal identifier"),
    ),
    responses(
        (status = 204, description = "Deal deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Deal not found"),
        (status = 409, description = "The deal is linked to a quote (E_DEAL_HAS_QUOTE)"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DealPath { deal_id }: DealPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, DealFailure> {
    let current = state.usecase.get_deal(deal_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_deal(deal_id, actor)
        .await??;

    Ok(Response::NoContent)
}
