use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{CreateThreadCommand, MessageId};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ChannelThreadsPath, require_permission, response::ChannelResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateThreadRequest {
    pub name: String,
    pub origin_message_id: Option<MessageId>,
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/channels/{channel_id}/threads",
    operation_id = "createThread",
    tag = super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Parent TEXT channel")),
    request_body = CreateThreadRequest,
    responses(
        (status = 201, description = "Thread created", body = inline(DataEnvelope<ChannelResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
        (status = 404, description = "Parent channel not found"),
        (status = 409, description = "Parent is not a TEXT channel"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelThreadsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateThreadRequest>,
) -> Result<Response<ChannelResponse>, ApiError> {
    let parent = state.usecase.get_channel(path.channel_id).await?;
    require_permission(&state, &identity, parent.organization_id, "channel.manage").await?;

    let thread = state
        .usecase
        .create_thread(CreateThreadCommand {
            organization_id: parent.organization_id,
            parent_id: path.channel_id,
            origin_message_id: payload.origin_message_id,
            name: payload.name,
        })
        .await?;

    Ok(Response::Created(thread.into()))
}
