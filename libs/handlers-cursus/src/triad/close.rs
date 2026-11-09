use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CloseTriadCommand;

use crate::{
    paths::TriadClosePath, request::CloseRequest, require_org_membership,
    response::TriadDetailResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}/close",
    operation_id = "closeCursusTriad",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("triad_id" = mestier_core::TriadId, Path, description = "Triad identifier"),
    ),
    request_body = CloseRequest,
    responses(
        (status = 200, description = "The triad and its open contracts are closed", body = inline(DataEnvelope<TriadDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Triad or closure reason not found in the organization"),
        (status = 409, description = "The triad is already closed or the closure reason is archived"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TriadClosePath {
        organization_id,
        triad_id,
    }: TriadClosePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CloseRequest>,
) -> Result<Response<TriadDetailResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let closed = state
        .usecase
        .acting_as(user_id)
        .close_triad(CloseTriadCommand {
            actor,
            organization_id,
            id: triad_id,
            closed_at: payload.closed_at,
            closure_reason_id: payload.closure_reason_id,
            closure_note: payload.closure_note,
        })
        .await?;

    Ok(Response::OK(closed.into()))
}
