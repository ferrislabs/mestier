use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{CategoryId, UpdateChannelCommand};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ChannelPath, require_permission, response::ChannelResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateChannelRequest {
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
    pub category_id: Option<CategoryId>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/chat/channels/{channel_id}",
    operation_id = "updateChannel",
    tag = super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
    request_body = UpdateChannelRequest,
    responses(
        (status = 200, description = "Channel updated", body = inline(DataEnvelope<ChannelResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateChannelRequest>,
) -> Result<Response<ChannelResponse>, ApiError> {
    // Fetch the channel first to obtain organization_id for the permission check.
    let existing = state.usecase.get_channel(path.channel_id).await?;

    require_permission(
        &state,
        &identity,
        existing.organization_id,
        "channel.manage",
    )
    .await?;

    if payload.name.trim().is_empty() {
        return Err(ApiError::Validation(
            "channel name must not be blank".into(),
        ));
    }

    let updated = state
        .usecase
        .update_channel(UpdateChannelCommand {
            id: path.channel_id,
            category_id: payload.category_id,
            name: payload.name,
            topic: payload.topic,
            position: payload.position,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
