use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::UpdateThreadCommand;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ThreadPath, require_permission, response::ChannelResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateThreadRequest {
    pub name: String,
    pub archived: bool,
}

#[utoipa::path(
    patch,
    path = "/api/v1/chat/threads/{channel_id}",
    operation_id = "updateThread",
    tag = super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Thread identifier")),
    request_body = UpdateThreadRequest,
    responses(
        (status = 200, description = "Thread updated", body = inline(DataEnvelope<ChannelResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
        (status = 404, description = "Thread not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ThreadPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateThreadRequest>,
) -> Result<Response<ChannelResponse>, ApiError> {
    let existing = state.usecase.get_channel(path.channel_id).await?;
    require_permission(
        &state,
        &identity,
        existing.organization_id,
        "channel.manage",
    )
    .await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(actor)
        .update_thread(UpdateThreadCommand {
            id: path.channel_id,
            name: payload.name,
            archived: payload.archived,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
