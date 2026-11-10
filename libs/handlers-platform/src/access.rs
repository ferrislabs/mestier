use auth::Identity;
use axum::Extension;
use handlers::{ApiError, DataEnvelope, Response};

use crate::{AccessPath, news_editor, response::NewsAccessResponse};

#[utoipa::path(
    get,
    path = "/api/v1/news/access",
    operation_id = "getNewsAccess",
    tag = crate::TAG,
    responses(
        (status = 200, description = "Whether the caller may use the news back-office", body = inline(DataEnvelope<NewsAccessResponse>)),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: AccessPath,
    Extension(identity): Extension<Identity>,
) -> Result<Response<NewsAccessResponse>, ApiError> {
    let can_manage = news_editor(&identity).is_ok();

    Ok(Response::OK(NewsAccessResponse { can_manage }))
}
