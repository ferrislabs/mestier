use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};

use crate::{paths::MyTriadPath, response::MyTriadDetailResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/me/triads/{triad_id}",
    operation_id = "getMyCursusTriad",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("triad_id" = mestier_core::TriadId, Path, description = "Triad identifier"),
    ),
    responses(
        (status = 200, description = "The triad with its promotion, contracts and the caller's role and access", body = inline(DataEnvelope<MyTriadDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "The caller is not an active participant of the organization"),
        (status = 404, description = "Triad outside the caller's scope"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MyTriadPath {
        organization_id,
        triad_id,
    }: MyTriadPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MyTriadDetailResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;

    let found = state
        .usecase
        .get_my_triad(user_id, organization_id, triad_id)
        .await?;

    Ok(Response::OK(found.into()))
}
