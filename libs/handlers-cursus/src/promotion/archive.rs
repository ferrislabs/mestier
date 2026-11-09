use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{paths::PromotionArchivePath, require_org_membership, response::PromotionResponse};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/promotions/{promotion_id}/archive",
    operation_id = "archiveCursusPromotion",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("promotion_id" = mestier_core::PromotionId, Path, description = "Promotion identifier"),
    ),
    responses(
        (status = 200, description = "The promotion is archived", body = inline(DataEnvelope<PromotionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Promotion not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PromotionArchivePath {
        organization_id,
        promotion_id,
    }: PromotionArchivePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<PromotionResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let archived = state
        .usecase
        .acting_as(user_id)
        .archive_promotion(actor, organization_id, promotion_id)
        .await?;

    Ok(Response::OK(archived.into()))
}
