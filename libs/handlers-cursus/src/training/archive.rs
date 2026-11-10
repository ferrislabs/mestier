use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{paths::TrainingArchivePath, require_org_membership, response::TrainingResponse};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/trainings/{training_id}/archive",
    operation_id = "archiveCursusTraining",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("training_id" = mestier_core::TrainingId, Path, description = "Training identifier"),
    ),
    responses(
        (status = 200, description = "The training is archived", body = inline(DataEnvelope<TrainingResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Training not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TrainingArchivePath {
        organization_id,
        training_id,
    }: TrainingArchivePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<TrainingResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let archived = state
        .usecase
        .acting_as(user_id)
        .archive_training(actor, organization_id, training_id)
        .await?;

    Ok(Response::OK(archived.into()))
}
