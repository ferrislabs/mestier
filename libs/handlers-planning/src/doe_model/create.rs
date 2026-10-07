use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreateDoeModelCommand, DoeModelSection};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    doe_model::{DoeModelsPath, response::DoeModelResponse},
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDoeModelRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub sections: Vec<DoeModelSection>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/doe-models",
    operation_id = "createDoeModel",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateDoeModelRequest,
    responses(
        (status = 201, description = "DOE model created", body = inline(DataEnvelope<DoeModelResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "A blank name or title, no section, or too many sections"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DoeModelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateDoeModelRequest>,
) -> Result<Response<DoeModelResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let model = state
        .usecase
        .acting_as(user_id)
        .create_doe_model(CreateDoeModelCommand {
            actor,
            organization_id: path.organization_id,
            name: payload.name,
            description: payload.description,
            sections: payload.sections,
        })
        .await?;

    Ok(Response::Created(model.into()))
}
