use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{TrainingId, UpdatePromotionCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::PromotionPath, require_org_membership, response::PromotionResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePromotionRequest {
    pub training_id: TrainingId,
    pub label: String,
    pub start_year: i32,
    pub end_year: i32,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/cursus/promotions/{promotion_id}",
    operation_id = "updateCursusPromotion",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("promotion_id" = mestier_core::PromotionId, Path, description = "Promotion identifier"),
    ),
    request_body = UpdatePromotionRequest,
    responses(
        (status = 200, description = "The promotion was updated", body = inline(DataEnvelope<PromotionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Promotion not found in the organization"),
        (status = 409, description = "Invalid promotion"),
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
    Json(payload): Json<UpdatePromotionRequest>,
) -> Result<Response<PromotionResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .update_promotion(UpdatePromotionCommand {
            actor,
            organization_id,
            id: promotion_id,
            training_id: payload.training_id,
            label: payload.label,
            start_year: payload.start_year,
            end_year: payload.end_year,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
