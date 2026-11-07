use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{CreateDealCommand, CustomerId, LeadSource, MemberId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    DEALS_TAG, deal_error::DealFailure, paths::DealsPath, require_org_membership,
    response::DealResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDealRequest {
    pub customer_id: CustomerId,
    pub title: String,
    #[serde(default)]
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
    post,
    path = "/api/v1/organizations/{organization_id}/deals",
    operation_id = "createDeal",
    tag = DEALS_TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateDealRequest,
    responses(
        (status = 201, description = "Deal created", body = inline(DataEnvelope<DealResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
        (status = 422, description = "A field is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DealsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateDealRequest>,
) -> Result<Response<DealResponse>, DealFailure> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let deal = state
        .usecase
        .acting_as(user_id)
        .create_deal(CreateDealCommand {
            actor,
            organization_id: path.organization_id,
            customer_id: payload.customer_id,
            title: payload.title,
            amount_cents: payload.amount_cents,
            expected_close_date: payload.expected_close_date,
            owner_member_id: payload.owner_member_id,
            source: payload.source,
            notes: payload.notes,
        })
        .await??;

    Ok(Response::Created(deal.into()))
}
