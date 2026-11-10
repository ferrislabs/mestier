use auth::Identity;
use axum::{Extension, body::Body, extract::State, response::IntoResponse};
use handlers::{ApiError, AppState, require_org_membership, resolve_actor};
use http::{
    StatusCode,
    header::{CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS},
};
use mestier_core::GetKnowledgeCommand;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

use crate::DownloadPath;

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}/download",
    operation_id = "downloadKnowledgeFile",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("article_id" = mestier_core::KnowledgeArticleId, Path, description = "Article identifier"),
    ),
    responses(
        (status = 200, description = "The stored file, as an attachment", content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Article not found, held by another organization, or a note"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DownloadPath {
        organization_id,
        article_id,
    }: DownloadPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<impl IntoResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let found = state
        .usecase
        .get_knowledge_article(GetKnowledgeCommand {
            actor,
            organization_id,
            id: article_id,
        })
        .await?;
    let file = found.article.file.ok_or(ApiError::NotFound)?;
    let object = state.file_storage.get(&file.key).await?;

    let name = utf8_percent_encode(&found.article.title, NON_ALPHANUMERIC);
    Ok((
        StatusCode::OK,
        [
            (CONTENT_TYPE.as_str(), file.mime_type),
            (
                CONTENT_DISPOSITION.as_str(),
                format!("attachment; filename*=UTF-8''{name}"),
            ),
            (X_CONTENT_TYPE_OPTIONS.as_str(), "nosniff".to_owned()),
            (CACHE_CONTROL.as_str(), "private, max-age=300".to_owned()),
        ],
        Body::from(object.bytes),
    ))
}
