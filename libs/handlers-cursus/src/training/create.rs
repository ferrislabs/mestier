use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreateTrainingCommand, EstablishmentId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::TrainingCollectionPath, require_org_membership, response::TrainingResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTrainingRequest {
    pub establishment_id: Option<EstablishmentId>,
    pub name: String,
    pub rncp_code: Option<String>,
    pub rncp_label: Option<String>,
    pub level: Option<String>,
    pub degree_type: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/trainings",
    operation_id = "createCursusTraining",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateTrainingRequest,
    responses(
        (status = 201, description = "The training was created", body = inline(DataEnvelope<TrainingResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Referenced resource not found in the organization"),
        (status = 409, description = "Invalid training"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TrainingCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateTrainingRequest>,
) -> Result<Response<TrainingResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let created = state
        .usecase
        .acting_as(user_id)
        .create_training(CreateTrainingCommand {
            actor,
            organization_id: path.organization_id,
            establishment_id: payload.establishment_id,
            name: payload.name,
            rncp_code: payload.rncp_code,
            rncp_label: payload.rncp_label,
            level: payload.level,
            degree_type: payload.degree_type,
        })
        .await?;

    Ok(Response::Created(created.into()))
}
