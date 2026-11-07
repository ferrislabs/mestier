use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{DealId, DealStageName, MoveDealCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    DEALS_TAG, deal_error::DealFailure, paths::DealStagePath, require_org_membership,
    response::DealResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct MoveDealRequest {
    pub stage: DealStageName,
    #[serde(default)]
    pub lost_reason: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/deals/{deal_id}/stage",
    operation_id = "moveDealStage",
    tag = DEALS_TAG,
    params(
        ("deal_id" = DealId, Path, description = "Deal identifier"),
    ),
    request_body = MoveDealRequest,
    responses(
        (status = 200, description = "Deal moved", body = inline(DataEnvelope<DealResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Deal not found"),
        (status = 422, description = "Moving to LOST needs a reason (E_DEAL_LOST_REASON_REQUIRED)"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DealStagePath { deal_id }: DealStagePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<MoveDealRequest>,
) -> Result<Response<DealResponse>, DealFailure> {
    let current = state.usecase.get_deal(deal_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let deal = state
        .usecase
        .acting_as(user_id)
        .move_deal(MoveDealCommand {
            actor,
            id: deal_id,
            stage: payload.stage,
            lost_reason: payload.lost_reason,
        })
        .await??;

    Ok(Response::OK(deal.into()))
}
