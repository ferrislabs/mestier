use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{
    paths::EstablishmentArchivePath, require_org_membership, response::EstablishmentResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/establishments/{establishment_id}/archive",
    operation_id = "archiveCursusEstablishment",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("establishment_id" = mestier_core::EstablishmentId, Path, description = "Establishment identifier"),
    ),
    responses(
        (status = 200, description = "The establishment is archived", body = inline(DataEnvelope<EstablishmentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Establishment not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    EstablishmentArchivePath {
        organization_id,
        establishment_id,
    }: EstablishmentArchivePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EstablishmentResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let archived = state
        .usecase
        .acting_as(user_id)
        .archive_establishment(actor, organization_id, establishment_id)
        .await?;

    Ok(Response::OK(archived.into()))
}
