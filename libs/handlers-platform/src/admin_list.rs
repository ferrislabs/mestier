use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{AdminNewsPath, platform_admin, response::NewsResponse};

#[utoipa::path(
    get,
    path = "/api/v1/admin/news",
    operation_id = "listNewsForAdmin",
    tag = crate::TAG,
    responses(
        (status = 200, description = "Every news, drafts and scheduled ones included, most recently updated first", body = inline(DataEnvelope<Vec<NewsResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: AdminNewsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<NewsResponse>>, ApiError> {
    let admin = platform_admin(&identity)?;
    let (user_id, _) = resolve_actor(&state, &identity).await?;

    let news = state
        .usecase
        .acting_as(user_id)
        .list_news_for_admin(admin)
        .await?;

    Ok(Response::OK(news.into_iter().map(Into::into).collect()))
}
