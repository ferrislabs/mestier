use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{EstablishmentId, UpdateTrainingCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::TrainingPath, require_org_membership, response::TrainingResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTrainingRequest {
    pub establishment_id: Option<EstablishmentId>,
    pub name: String,
    pub rncp_code: Option<String>,
    pub rncp_label: Option<String>,
    pub level: Option<String>,
    pub degree_type: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/cursus/trainings/{training_id}",
    operation_id = "updateCursusTraining",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("training_id" = mestier_core::TrainingId, Path, description = "Training identifier"),
    ),
    request_body = UpdateTrainingRequest,
    responses(
        (status = 200, description = "The training was updated", body = inline(DataEnvelope<TrainingResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Training not found in the organization"),
        (status = 409, description = "Invalid training"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TrainingPath {
        organization_id,
        training_id,
    }: TrainingPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateTrainingRequest>,
) -> Result<Response<TrainingResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .update_training(UpdateTrainingCommand {
            actor,
            organization_id,
            id: training_id,
            establishment_id: payload.establishment_id,
            name: payload.name,
            rncp_code: payload.rncp_code,
            rncp_label: payload.rncp_label,
            level: payload.level,
            degree_type: payload.degree_type,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
