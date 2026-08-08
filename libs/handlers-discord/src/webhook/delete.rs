use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};

use crate::{EmptyResponse, paths::WebhookPath, require_permission};

#[utoipa::path(
	delete,
	path = "/api/v1/chat/webhooks/{webhook_id}",
	operation_id = "deleteWebhook",
	tag = super::super::TAG,
	params(("webhook_id" = discord::WebhookId, Path, description = "Webhook identifier")),
	responses(
		(status = 204, description = "Webhook deleted"),
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
) -> Result<Response<EmptyResponse>, ApiError> {
    let webhook = state.usecase.get_webhook(path.webhook_id).await?;
    require_permission(&state, &identity, webhook.organization_id, "webhook.manage").await?;
    let actor = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .acting_as(actor)
        .delete_webhook(path.webhook_id)
        .await?;
    Ok(Response::NoContent)
}
