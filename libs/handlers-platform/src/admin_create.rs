use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateNewsCommand;

use crate::{
    AdminNewsPath, platform_admin, request::NewsRequest, response::NewsResponse, unprocessable,
};

#[utoipa::path(
    post,
    path = "/api/v1/admin/news",
    operation_id = "createNews",
    tag = crate::TAG,
    request_body = NewsRequest,
    responses(
        (status = 201, description = "The news", body = inline(DataEnvelope<NewsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Invalid content: title, items, button label or link"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: AdminNewsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<NewsRequest>,
) -> Result<Response<NewsResponse>, ApiError> {
    let admin = platform_admin(&identity)?;
    let (user_id, _) = resolve_actor(&state, &identity).await?;
    let (content, published_at) = payload.into_parts();

    let news = state
        .usecase
        .acting_as(user_id)
        .create_news(CreateNewsCommand {
            admin,
            content,
            published_at,
        })
        .await?
        .map_err(unprocessable)?;

    Ok(Response::Created(news.into()))
}
