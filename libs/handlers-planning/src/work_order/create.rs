use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateWorkOrderCommand, CustomerContextId, CustomerId, QuoteId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    require_org_membership,
    response::WorkOrderResponse,
    work_order::{WorkOrdersPath, require_work_order_targets},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWorkOrderRequest {
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    #[serde(default)]
    pub quote_id: Option<QuoteId>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/work-orders",
    operation_id = "createWorkOrder",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateWorkOrderRequest,
    responses(
        (status = 201, description = "Work order created", body = inline(DataEnvelope<WorkOrderResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Work order conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: WorkOrdersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateWorkOrderRequest>,
) -> Result<Response<WorkOrderResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    require_work_order_targets(
        &state,
        path.organization_id,
        payload.customer_id,
        payload.customer_context_id,
        payload.quote_id,
    )
    .await?;

    let work_order = state
        .usecase
        .create_work_order(CreateWorkOrderCommand {
            organization_id: path.organization_id,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            quote_id: payload.quote_id,
            starts_at: payload.starts_at,
            ends_at: payload.ends_at,
            all_day: payload.all_day,
            title: payload.title,
            note: payload.note,
        })
        .await?;

    Ok(Response::Created(work_order.into()))
}
