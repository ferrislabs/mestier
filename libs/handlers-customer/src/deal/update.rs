use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{CustomerId, DealId, LeadSource, MemberId, UpdateDealCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    DEALS_TAG, deal_error::DealFailure, paths::DealPath, require_org_membership,
    response::DealResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDealRequest {
    pub customer_id: CustomerId,
    pub title: String,
    pub amount_cents: i64,
    #[serde(default)]
    pub expected_close_date: Option<NaiveDate>,
    #[serde(default)]
    pub owner_member_id: Option<MemberId>,
    #[serde(default)]
    pub source: Option<LeadSource>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/deals/{deal_id}",
    operation_id = "updateDeal",
    tag = DEALS_TAG,
    params(
        ("deal_id" = DealId, Path, description = "Deal identifier"),
    ),
    request_body = UpdateDealRequest,
    responses(
        (status = 200, description = "Deal updated", body = inline(DataEnvelope<DealResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Deal or customer not found"),
        (status = 422, description = "A field is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DealPath { deal_id }: DealPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateDealRequest>,
) -> Result<Response<DealResponse>, DealFailure> {
    let current = state.usecase.get_deal(deal_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let deal = state
        .usecase
        .acting_as(user_id)
        .update_deal(UpdateDealCommand {
            actor,
            id: deal_id,
            customer_id: payload.customer_id,
            title: payload.title,
            amount_cents: payload.amount_cents,
            expected_close_date: payload.expected_close_date,
            owner_member_id: payload.owner_member_id,
            source: payload.source,
            notes: payload.notes,
        })
        .await??;

    Ok(Response::OK(deal.into()))
}
