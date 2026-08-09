use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{CreateMemberCommand, application::policy};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::MembersPath, response::MemberResponse};

/// A free, named seat — no `user_id`. Occupying it is an invitation concern
/// (#184), not part of creating the seat itself.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMemberRequest {
    pub last_name: String,
    /// `null` means "not provided" — see `Member::first_name`.
    #[serde(default)]
    pub first_name: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/members",
    operation_id = "createMember",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateMemberRequest,
    responses(
        (status = 201, description = "Member created", body = inline(DataEnvelope<MemberResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Member conflict"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(organization_id = %path.organization_id.0), err)]
pub async fn handler(
    path: MembersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateMemberRequest>,
) -> Result<Response<MemberResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;
    // TODO: thread JWT realm roles once Identity exposes them.
    let actor = policy::user_subject(user_id, Vec::new());

    let member = state
        .usecase
        .acting_as(user_id)
        .create_member(CreateMemberCommand {
            actor,
            organization_id: path.organization_id,
            last_name: payload.last_name,
            first_name: payload.first_name,
        })
        .await?;

    // A freshly created seat is always vacant: `account` stays `None`, no
    // lookup needed.
    Ok(Response::Created(member.into()))
}
