use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};

use crate::{EmptyResponse, paths::ThreadPath, require_permission};

#[utoipa::path(
    delete,
    path = "/api/v1/threads/{channel_id}",
    operation_id = "deleteThread",
    tag = super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Thread identifier")),
    responses(
        (status = 204, description = "Thread deleted"),
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
) -> Result<Response<EmptyResponse>, ApiError> {
    let existing = state.usecase.get_channel(path.channel_id).await?;
    require_permission(
        &state,
        &identity,
        existing.organization_id,
        "channel.manage",
    )
    .await?;
    state.usecase.delete_thread(path.channel_id).await?;
    Ok(Response::NoContent)
}
