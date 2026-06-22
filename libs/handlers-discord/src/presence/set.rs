use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{PresenceStatus, SetPresenceCommand};
use handlers::{ApiError, AppState, DataEnvelope, IdentityExt, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::OrgPresencePath, require_org_membership, response::PresenceResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetPresenceRequest {
    pub status: PresenceStatus,
}

#[utoipa::path(
	put,
	path = "/api/v1/chat/organizations/{organization_id}/presence",
	operation_id = "setPresence",
	tag = super::super::TAG,
	params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
	request_body = SetPresenceRequest,
	responses(
		(status = 200, description = "Presence updated", body = inline(DataEnvelope<PresenceResponse>)),
		(status = 400, description = "Invalid status"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgPresencePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SetPresenceRequest>,
) -> Result<Response<PresenceResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let user_id = identity.user_id()?;

    let presence = state
        .usecase
        .set_presence(SetPresenceCommand {
            organization_id: path.organization_id,
            user_id,
            status: payload.status,
        })
        .await?;

    Ok(Response::OK(presence.into()))
}
