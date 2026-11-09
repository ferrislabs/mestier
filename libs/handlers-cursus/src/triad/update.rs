use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateTriadCommand;

use crate::{
    paths::TriadPath, request::UpdateTriadRequest, require_org_membership,
    response::TriadDetailResponse,
};

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}",
    operation_id = "updateCursusTriad",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("triad_id" = mestier_core::TriadId, Path, description = "Triad identifier"),
    ),
    request_body = UpdateTriadRequest,
    responses(
        (status = 200, description = "The triad was updated", body = inline(DataEnvelope<TriadDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Triad or pedagogic tutor not found in the organization"),
        (status = 409, description = "The triad is closed"),
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
    Json(payload): Json<UpdateTriadRequest>,
) -> Result<Response<TriadDetailResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .update_triad(UpdateTriadCommand {
            actor,
            organization_id,
            id: triad_id,
            pedagogic_tutor_member_id: payload.pedagogic_tutor_member_id,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
