use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::EstablishmentPath, require_view_cursus, response::EstablishmentResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/establishments/{establishment_id}",
    operation_id = "getCursusEstablishment",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("establishment_id" = mestier_core::EstablishmentId, Path, description = "Establishment identifier"),
    ),
    responses(
        (status = 200, description = "The establishment", body = inline(DataEnvelope<EstablishmentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Establishment not found in the organization"),
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
) -> Result<Response<EstablishmentResponse>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let found = state
        .usecase
        .get_establishment(organization_id, establishment_id)
        .await?;

    Ok(Response::OK(found.into()))
}
