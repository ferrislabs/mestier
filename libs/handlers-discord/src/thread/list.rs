use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::ChannelThreadsPath, require_org_membership, response::ChannelResponse};

#[utoipa::path(
    get,
    path = "/api/v1/chat/channels/{channel_id}/threads",
    operation_id = "listThreads",
    tag = super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Parent TEXT channel")),
    responses(
        (status = 200, description = "List of threads", body = inline(DataEnvelope<Vec<ChannelResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Parent channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelThreadsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<ChannelResponse>>, ApiError> {
    let parent = state.usecase.get_channel(path.channel_id).await?;
    require_org_membership(&state, &identity, parent.organization_id).await?;

    let threads = state.usecase.list_threads(path.channel_id).await?;
    let items: Vec<ChannelResponse> = threads.into_iter().map(ChannelResponse::from).collect();
    Ok(Response::OK(items))
}
