use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateNewsCommand;

use crate::{
    AdminNewsItemPath, news_editor, request::NewsRequest, response::NewsResponse, unprocessable,
};

#[utoipa::path(
    put,
    path = "/api/v1/admin/news/{news_id}",
    operation_id = "updateNews",
    tag = crate::TAG,
    params(("news_id" = mestier_core::NewsId, Path, description = "News identifier")),
    request_body = NewsRequest,
    responses(
        (status = 200, description = "The news, replaced", body = inline(DataEnvelope<NewsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "News not found"),
        (status = 422, description = "Invalid content: title, items, button label or link"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AdminNewsItemPath { news_id }: AdminNewsItemPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<NewsRequest>,
) -> Result<Response<NewsResponse>, ApiError> {
    let editor = news_editor(&identity)?;
    let (user_id, _) = resolve_actor(&state, &identity).await?;
    let (content, published_at) = payload.into_parts();

    let news = state
        .usecase
        .acting_as(user_id)
        .update_news(UpdateNewsCommand {
            editor,
            id: news_id,
            content,
            published_at,
        })
        .await?
        .map_err(unprocessable)?;

    Ok(Response::OK(news.into()))
}
