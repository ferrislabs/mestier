use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::PromotionPath, require_view_cursus, response::PromotionResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/promotions/{promotion_id}",
    operation_id = "getCursusPromotion",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("promotion_id" = mestier_core::PromotionId, Path, description = "Promotion identifier"),
    ),
    responses(
        (status = 200, description = "The promotion", body = inline(DataEnvelope<PromotionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Promotion not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PromotionPath {
        organization_id,
        promotion_id,
    }: PromotionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<PromotionResponse>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let found = state
        .usecase
        .get_promotion(organization_id, promotion_id)
        .await?;

    Ok(Response::OK(found.into()))
}
