use auth::Identity;
use axum::{Extension, extract::Query, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, PaginationParams, Response};

use crate::{paths::DeliveriesPath, require_org_membership, response::DeliveryResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/deliveries",
    operation_id = "listAutomationDeliveries",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("page" = Option<u64>, Query, description = "Page number, 1-based"),
        ("per_page" = Option<u64>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Delivery log, newest first", body = inline(DataEnvelope<Vec<DeliveryResponse>>)),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DeliveriesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<Vec<DeliveryResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let (deliveries, _total) = state
        .usecase
        .list_deliveries(
            path.organization_id,
            pagination.per_page() as i64,
            ((pagination.page() - 1) * pagination.per_page()) as i64,
        )
        .await?;

    Ok(Response::OK(
        deliveries.into_iter().map(DeliveryResponse::from).collect(),
    ))
}
