use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Duration, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{InviteMemberCommand, MemberId, application::policy};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::InvitationsPath, response::CreatedInvitationResponse};

/// Default validity when the caller does not specify one — long enough for a
/// link handed out in person or over chat to still work a week later.
const DEFAULT_TTL_DAYS: i64 = 7;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInvitationRequest {
    /// `None` — nobody has a seat yet; acceptance creates one. `Some` —
    /// grant login access to that seat. See `Invitation::member_id`.
    #[serde(default)]
    pub member_id: Option<MemberId>,
    /// `None` defaults to seven days from now.
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

/// A free, named seat — no `user_id`. Occupying it is an invitation concern
/// (#184), not part of creating the seat itself.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/invitations",
    operation_id = "createInvitation",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateInvitationRequest,
    responses(
        (status = 201, description = "Invitation created — the token is visible in this response and never again", body = inline(DataEnvelope<CreatedInvitationResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Target member not found"),
        (status = 409, description = "Target seat already occupied, or expiry not in the future"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(organization_id = %path.organization_id.0), err)]
pub async fn handler(
    path: InvitationsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<Response<CreatedInvitationResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;
    // TODO: thread JWT realm roles once Identity exposes them.
    let actor = policy::user_subject(user_id, Vec::new());

    let expires_at = payload
        .expires_at
        .unwrap_or_else(|| Utc::now() + Duration::days(DEFAULT_TTL_DAYS));

    let (invitation, token) = state
        .usecase
        .acting_as(user_id)
        .invite_member(InviteMemberCommand {
            actor,
            organization_id: path.organization_id,
            member_id: payload.member_id,
            expires_at,
        })
        .await?;

    Ok(Response::Created(CreatedInvitationResponse {
        invitation: invitation.into(),
        token,
    }))
}
