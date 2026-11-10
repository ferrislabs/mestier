use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateEstablishmentCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::EstablishmentCollectionPath, require_org_membership, response::EstablishmentResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEstablishmentRequest {
    pub name: String,
    pub uai: Option<String>,
    pub siret: Option<String>,
    pub address: Option<String>,
    #[serde(default)]
    pub is_main_site: bool,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/establishments",
    operation_id = "createCursusEstablishment",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateEstablishmentRequest,
    responses(
        (status = 201, description = "The establishment was created", body = inline(DataEnvelope<EstablishmentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Referenced resource not found in the organization"),
        (status = 409, description = "Invalid establishment"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EstablishmentCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateEstablishmentRequest>,
) -> Result<Response<EstablishmentResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let created = state
        .usecase
        .acting_as(user_id)
        .create_establishment(CreateEstablishmentCommand {
            actor,
            organization_id: path.organization_id,
            name: payload.name,
            uai: payload.uai,
            siret: payload.siret,
            address: payload.address,
            is_main_site: payload.is_main_site,
        })
        .await?;

    Ok(Response::Created(created.into()))
}
