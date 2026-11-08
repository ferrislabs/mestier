use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{CategoryId, ChannelType, CreateChannelCommand, RoleId};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::OrgChannelsPath, require_permission, response::ChannelResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateChannelRequest {
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
    pub category_id: Option<CategoryId>,
    /// `TEXT` when absent. Only `TEXT`, `ANNOUNCEMENT` and `VOICE` are accepted.
    pub channel_type: Option<ChannelType>,
    /// Present means private: only members holding one of these roles, and the
    /// creator, can see and write in the channel.
    pub allowed_role_ids: Option<Vec<RoleId>>,
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/organizations/{organization_id}/channels",
    operation_id = "createChannel",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateChannelRequest,
    responses(
        (status = 201, description = "Channel created", body = inline(DataEnvelope<ChannelResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
        (status = 409, description = "Unsupported channel type, or unknown or empty role list"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgChannelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<Response<ChannelResponse>, ApiError> {
    require_permission(&state, &identity, path.organization_id, "channel.manage").await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let command = CreateChannelCommand {
        organization_id: path.organization_id,
        channel_type: payload.channel_type.unwrap_or(ChannelType::Text),
        category_id: payload.category_id,
        name: payload.name,
        topic: payload.topic,
        position: payload.position,
    };
    let usecase = state.usecase.acting_as(actor);
    let channel = match payload.allowed_role_ids {
        Some(role_ids) => {
            usecase
                .create_private_channel(command, actor, role_ids)
                .await?
        }
        None => usecase.create_channel(command).await?,
    };

    Ok(Response::Created(channel.into()))
}
