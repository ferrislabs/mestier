use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateEstablishmentCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::EstablishmentPath, require_org_membership, response::EstablishmentResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEstablishmentRequest {
    pub name: String,
    pub uai: Option<String>,
    pub siret: Option<String>,
    pub address: Option<String>,
    #[serde(default)]
    pub is_main_site: bool,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/cursus/establishments/{establishment_id}",
    operation_id = "updateCursusEstablishment",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("establishment_id" = mestier_core::EstablishmentId, Path, description = "Establishment identifier"),
    ),
    request_body = UpdateEstablishmentRequest,
    responses(
        (status = 200, description = "The establishment was updated", body = inline(DataEnvelope<EstablishmentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Establishment not found in the organization"),
        (status = 409, description = "Invalid establishment"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    EstablishmentPath {
        organization_id,
        establishment_id,
    }: EstablishmentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateEstablishmentRequest>,
) -> Result<Response<EstablishmentResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .update_establishment(UpdateEstablishmentCommand {
            actor,
            organization_id,
            id: establishment_id,
            name: payload.name,
            uai: payload.uai,
            siret: payload.siret,
            address: payload.address,
            is_main_site: payload.is_main_site,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
