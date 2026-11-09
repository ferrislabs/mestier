use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreatePromotionCommand, TrainingId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::PromotionCollectionPath, require_org_membership, response::PromotionResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePromotionRequest {
    pub training_id: TrainingId,
    pub label: String,
    pub start_year: i32,
    pub end_year: i32,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/promotions",
    operation_id = "createCursusPromotion",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreatePromotionRequest,
    responses(
        (status = 201, description = "The promotion was created", body = inline(DataEnvelope<PromotionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Referenced resource not found in the organization"),
        (status = 409, description = "Invalid promotion"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: PromotionCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreatePromotionRequest>,
) -> Result<Response<PromotionResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let created = state
        .usecase
        .acting_as(user_id)
        .create_promotion(CreatePromotionCommand {
            actor,
            organization_id: path.organization_id,
            training_id: payload.training_id,
            label: payload.label,
            start_year: payload.start_year,
            end_year: payload.end_year,
        })
        .await?;

    Ok(Response::Created(created.into()))
}
