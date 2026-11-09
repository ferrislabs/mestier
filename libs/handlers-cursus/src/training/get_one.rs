use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::TrainingPath, require_view_cursus, response::TrainingResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/trainings/{training_id}",
    operation_id = "getCursusTraining",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("training_id" = mestier_core::TrainingId, Path, description = "Training identifier"),
    ),
    responses(
        (status = 200, description = "The training", body = inline(DataEnvelope<TrainingResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Training not found in the organization"),
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
) -> Result<Response<TrainingResponse>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let found = state
        .usecase
        .get_training(organization_id, training_id)
        .await?;

    Ok(Response::OK(found.into()))
}
