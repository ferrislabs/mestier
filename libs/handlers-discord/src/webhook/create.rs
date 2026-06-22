use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::CreateWebhookCommand;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::ChannelWebhooksPath,
    require_permission, resolve_user_id,
    response::{WebhookCreatedResponse, webhook_created_response},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[utoipa::path(
	post,
	path = "/api/v1/chat/channels/{channel_id}/webhooks",
	operation_id = "createWebhook",
	tag = super::super::TAG,
	params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
	request_body = CreateWebhookRequest,
	responses(
		(status = 201, description = "Webhook created — token returned only here", body = inline(DataEnvelope<WebhookCreatedResponse>)),
		(status = 400, description = "Validation failed"),
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
    Json(payload): Json<CreateWebhookRequest>,
) -> Result<Response<WebhookCreatedResponse>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_permission(&state, &identity, channel.organization_id, "webhook.manage").await?;

    if payload.name.trim().is_empty() {
        return Err(ApiError::Validation(
            "webhook name must not be blank".into(),
        ));
    }

    let created_by = resolve_user_id(&state, &identity).await?;

    let webhook = state
        .usecase
        .create_webhook(CreateWebhookCommand {
            organization_id: channel.organization_id,
            channel_id: path.channel_id,
            name: payload.name,
            avatar_url: payload.avatar_url,
            created_by,
        })
        .await?;

    Ok(Response::Created(webhook_created_response(webhook)))
}
