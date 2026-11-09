use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::TriadPath, require_view_cursus, response::TriadDetailResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}",
    operation_id = "getCursusTriad",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("triad_id" = mestier_core::TriadId, Path, description = "Triad identifier"),
    ),
    responses(
        (status = 200, description = "The triad with its promotion, apprentice and contracts", body = inline(DataEnvelope<TriadDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Triad not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TriadPath {
        organization_id,
        triad_id,
    }: TriadPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<TriadDetailResponse>, ApiError> {
    require_view_cursus(&state, &identity, organization_id).await?;

    let found = state.usecase.get_triad(organization_id, triad_id).await?;

    Ok(Response::OK(found.into()))
}
