use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::ChannelWebhooksPath, require_permission, response::WebhookResponse};

#[utoipa::path(
	get,
	path = "/api/v1/chat/channels/{channel_id}/webhooks",
	operation_id = "listWebhooks",
	tag = super::super::TAG,
	params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
	responses(
		(status = 200, description = "Webhook list", body = inline(DataEnvelope<Vec<WebhookResponse>>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden — requires webhook.manage"),
		(status = 404, description = "Channel not found"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelWebhooksPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<WebhookResponse>>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_permission(&state, &identity, channel.organization_id, "webhook.manage").await?;

    let webhooks = state.usecase.list_webhooks(path.channel_id).await?;

    let items: Vec<WebhookResponse> = webhooks.into_iter().map(WebhookResponse::from).collect();
    Ok(Response::OK(items))
}
