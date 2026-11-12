use auth::Identity;
use axum::{Extension, extract::State, http::StatusCode};
use handlers::{ApiError, AppState, resolve_actor};

use crate::{AdminNewsItemPath, platform_admin};

#[utoipa::path(
    delete,
    path = "/api/v1/admin/news/{news_id}",
    operation_id = "deleteNews",
    tag = crate::TAG,
    params(("news_id" = mestier_core::NewsId, Path, description = "News identifier")),
    responses(
        (status = 204, description = "News and its views deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "News not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AdminNewsItemPath { news_id }: AdminNewsItemPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    let admin = platform_admin(&identity)?;
    let (user_id, _) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_news(admin, news_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
