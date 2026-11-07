use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::DealId;

use crate::{DEALS_TAG, paths::DealPath, require_view_customers, response::DealResponse};

#[utoipa::path(
    get,
    path = "/api/v1/deals/{deal_id}",
    operation_id = "getDeal",
    tag = DEALS_TAG,
    params(
        ("deal_id" = DealId, Path, description = "Deal identifier"),
    ),
    responses(
        (status = 200, description = "Deal details", body = inline(DataEnvelope<DealResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Deal not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DealPath { deal_id }: DealPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DealResponse>, ApiError> {
    let deal = state.usecase.get_deal(deal_id).await?;
    require_view_customers(&state, &identity, deal.organization_id).await?;

    Ok(Response::OK(deal.into()))
}
