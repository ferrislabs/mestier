use auth::Identity;
use axum::{Extension, Json, extract::State, http::StatusCode};
use handlers::{ApiError, AppState, resolve_user_id};
use mestier_core::check_seen_ids;

use crate::{SeenPath, request::MarkSeenRequest, unprocessable};

#[utoipa::path(
    post,
    path = "/api/v1/news/seen",
    operation_id = "markNewsSeen",
    tag = crate::TAG,
    request_body = MarkSeenRequest,
    responses(
        (status = 204, description = "Recorded for the caller. Idempotent; unknown or unpublished ids are ignored"),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "No id, or more than 50"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: SeenPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<MarkSeenRequest>,
) -> Result<StatusCode, ApiError> {
    check_seen_ids(&payload.ids).map_err(unprocessable)?;
    let user_id = resolve_user_id(&state, &identity).await?;

    state.usecase.mark_news_seen(user_id, payload.ids).await?;

    Ok(StatusCode::NO_CONTENT)
}
