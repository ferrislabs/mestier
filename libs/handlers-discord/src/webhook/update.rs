use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::UpdateWebhookCommand;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::WebhookPath, require_permission, response::WebhookResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWebhookRequest {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[utoipa::path(
	patch,
	path = "/api/v1/chat/webhooks/{webhook_id}",
	operation_id = "updateWebhook",
	tag = super::super::TAG,
	params(("webhook_id" = discord::WebhookId, Path, description = "Webhook identifier")),
	request_body = UpdateWebhookRequest,
	responses(
		(status = 200, description = "Webhook updated", body = inline(DataEnvelope<WebhookResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden — requires webhook.manage"),
		(status = 404, description = "Webhook not found"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: WebhookPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateWebhookRequest>,
) -> Result<Response<WebhookResponse>, ApiError> {
    let webhook = state.usecase.get_webhook(path.webhook_id).await?;
    require_permission(&state, &identity, webhook.organization_id, "webhook.manage").await?;

    if payload.name.trim().is_empty() {
        return Err(ApiError::Validation(
            "webhook name must not be blank".into(),
        ));
    }

    let updated = state
        .usecase
        .update_webhook(UpdateWebhookCommand {
            id: path.webhook_id,
            name: payload.name,
            avatar_url: payload.avatar_url,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
