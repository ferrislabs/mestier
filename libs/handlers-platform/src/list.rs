use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{NewsPath, response::NewsResponse};

#[utoipa::path(
    get,
    path = "/api/v1/news",
    operation_id = "listNews",
    tag = crate::TAG,
    responses(
        (status = 200, description = "Published news, most recently published first", body = inline(DataEnvelope<Vec<NewsResponse>>)),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: NewsPath,
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Response<Vec<NewsResponse>>, ApiError> {
    let news = state.usecase.list_published_news().await?;

    Ok(Response::OK(news.into_iter().map(Into::into).collect()))
}
