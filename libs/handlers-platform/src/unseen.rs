use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};

use crate::{UnseenPath, response::NewsResponse};

#[utoipa::path(
    get,
    path = "/api/v1/news/unseen",
    operation_id = "listUnseenNews",
    tag = crate::TAG,
    responses(
        (status = 200, description = "Published news the caller has not seen, most recently published first", body = inline(DataEnvelope<Vec<NewsResponse>>)),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: UnseenPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<NewsResponse>>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;

    let news = state.usecase.list_unseen_news(user_id).await?;

    Ok(Response::OK(news.into_iter().map(Into::into).collect()))
}
