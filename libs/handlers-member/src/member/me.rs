use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};

use crate::{paths::MyMemberPath, response::MyMemberResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/members/me",
    operation_id = "getMyMember",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The caller's own seat in this organization", body = inline(DataEnvelope<MyMemberResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "The caller holds no seat in this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MyMemberPath { organization_id }: MyMemberPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MyMemberResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;
    let member = state
        .usecase
        .find_membership(organization_id, user_id)
        .await?
        .ok_or_else(|| ApiError::ResourceNotFound("Member", organization_id.to_string()))?;

    Ok(Response::OK(member.into()))
}
