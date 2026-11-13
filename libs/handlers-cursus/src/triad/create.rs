use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateTriadCommand;

use crate::{
    paths::TriadCollectionPath, request::CreateTriadRequest, require_org_membership,
    response::TriadDetailResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/triads",
    operation_id = "createCursusTriad",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateTriadRequest,
    responses(
        (status = 201, description = "The triad was created", body = inline(DataEnvelope<TriadDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Promotion, apprentice or pedagogic tutor not found in the organization"),
        (status = 409, description = "The apprentice already has a triad in this promotion"),
        (status = 422, description = "`E_NOT_A_SCHOOL_TUTOR`: the pedagogic tutor is not an active school tutor of the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TriadCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateTriadRequest>,
) -> Result<Response<TriadDetailResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let usecase = state.usecase.acting_as(user_id);
    let created = usecase
        .create_triad(CreateTriadCommand {
            actor,
            organization_id: path.organization_id,
            promotion_id: payload.promotion_id,
            apprentice_id: payload.apprentice_id,
            pedagogic_tutor_member_id: payload.pedagogic_tutor_member_id,
        })
        .await?;
    let detail = usecase.get_triad(path.organization_id, created.id).await?;

    Ok(Response::Created(detail.into()))
}
