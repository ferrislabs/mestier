use axum::{Json, extract::State};
use discord::{ExecuteWebhookCommand, components::Component};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::WebhookExecutePath, response::MessageResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecuteWebhookRequest {
    pub content: String,
    pub components: Option<Vec<Component>>,
}

#[utoipa::path(
	post,
	path = "/api/v1/chat/webhooks/{webhook_id}/{token}",
	operation_id = "executeWebhook",
	tag = super::super::TAG,
	params(
		("webhook_id" = discord::WebhookId, Path, description = "Webhook identifier"),
		("token" = String, Path, description = "Webhook secret token"),
	),
	request_body = ExecuteWebhookRequest,
	responses(
		(status = 201, description = "Message posted via webhook", body = inline(DataEnvelope<MessageResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 403, description = "Invalid webhook token"),
		(status = 404, description = "Webhook not found"),
	),
	// No security scheme — authenticated by the webhook token in the path
)]
pub async fn handler(
    path: WebhookExecutePath,
    State(state): State<AppState>,
    Json(payload): Json<ExecuteWebhookRequest>,
) -> Result<Response<MessageResponse>, ApiError> {
    if payload.content.trim().is_empty() && payload.components.is_none() {
        return Err(ApiError::Validation(
            "webhook message must have content or components".into(),
        ));
    }

    let message = state
        .usecase
        .execute_webhook(ExecuteWebhookCommand {
            webhook_id: path.webhook_id,
            token: path.token,
            content: payload.content,
            components: payload.components,
        })
        .await?; // bad token -> CoreError::Forbidden -> ApiError::Forbidden (403)

    Ok(Response::Created(message.into()))
}
