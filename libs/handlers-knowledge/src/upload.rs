use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use auth::Identity;
use axum::{
    Extension, Json,
    body::Bytes,
    extract::{RawQuery, State},
    response::IntoResponse,
};
use handlers::{ApiError, AppState, Response, require_org_membership, resolve_actor};
use http::{HeaderMap, StatusCode, header::CONTENT_TYPE};
use mestier_core::{
    KnowledgeError, MAX_FILE_BYTES, UploadKnowledgeFileCommand, check_knowledge_file,
    extract_knowledge_text,
};
use tokio::sync::Semaphore;
use url::form_urlencoded;

use crate::{ArticlesPath, response::KnowledgeArticleResponse, unprocessable};

#[derive(Debug)]
pub enum UploadError {
    TooLarge,
    Api(ApiError),
}

impl From<ApiError> for UploadError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<common::CoreError> for UploadError {
    fn from(error: common::CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl IntoResponse for UploadError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::TooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(serde_json::json!({
                    "code": "E_PAYLOAD_TOO_LARGE",
                    "message": format!("file exceeds the maximum size of {MAX_FILE_BYTES} bytes"),
                    "status": 413,
                })),
            )
                .into_response(),
        }
    }
}

const CONCURRENT_EXTRACTIONS: usize = 4;
const EXTRACTION_TIMEOUT: Duration = Duration::from_secs(30);

static EXTRACTION_PERMITS: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(CONCURRENT_EXTRACTIONS)));

/// Reads the text of a file off the async executor. A file whose text cannot
/// be read, or takes too long to read, is stored without it.
async fn read_text(mime_type: String, bytes: Bytes) -> Option<String> {
    let permit = EXTRACTION_PERMITS.clone().acquire_owned().await.ok()?;
    let task = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        extract_knowledge_text(&mime_type, &bytes)
    });
    match tokio::time::timeout(EXTRACTION_TIMEOUT, task).await {
        Ok(Ok(text)) => text,
        Ok(Err(error)) => {
            tracing::warn!(error = %error, "text extraction task failed");
            None
        }
        Err(_) => {
            tracing::warn!("text extraction exceeded {EXTRACTION_TIMEOUT:?}");
            None
        }
    }
}

fn mime_type(headers: &HeaderMap) -> String {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_default()
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles",
    operation_id = "uploadKnowledgeFile",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("title" = String, Query, description = "Title of the article, usually the file name"),
        ("tag" = Option<Vec<String>>, Query, description = "Tag, repeatable"),
        ("folder_id" = Option<mestier_core::KnowledgeFolderId>, Query, description = "Folder to put the file in; absent puts it at the root"),
    ),
    request_body(content = Vec<u8>, content_type = "application/pdf"),
    responses(
        (status = 201, description = "The article", body = inline(handlers::DataEnvelope<KnowledgeArticleResponse>)),
        (status = 400, description = "The title is missing"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Folder not found, or held by another organization"),
        (status = 413, description = "The file exceeds 25 MB"),
        (status = 422, description = "Empty file, file type not accepted, or invalid title or tag"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ArticlesPath { organization_id }: ArticlesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    RawQuery(raw): RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<KnowledgeArticleResponse>, UploadError> {
    require_org_membership(&state, &identity, organization_id).await?;

    if body.len() > MAX_FILE_BYTES {
        return Err(UploadError::TooLarge);
    }
    let mut title = None;
    let mut tags = Vec::new();
    let mut folder_id = None;
    for (key, value) in form_urlencoded::parse(raw.unwrap_or_default().as_bytes()) {
        match key.as_ref() {
            "title" => title = Some(value.into_owned()),
            "tag" => tags.push(value.into_owned()),
            "folder_id" => {
                folder_id = Some(
                    value
                        .parse()
                        .map_err(|_| ApiError::Validation("folder_id: not a uuid".to_owned()))?,
                );
            }
            _ => {}
        }
    }
    let title = title.ok_or_else(|| ApiError::Validation("title: required".to_owned()))?;

    let mime_type = mime_type(&headers);
    check_knowledge_file(&mime_type, &body)
        .map_err(|error: KnowledgeError| unprocessable(error))?;
    let extracted_text = read_text(mime_type.clone(), body.clone()).await;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let article = state
        .usecase
        .acting_as(user_id)
        .upload_knowledge_file(
            UploadKnowledgeFileCommand {
                actor,
                organization_id,
                title,
                mime_type,
                bytes: body.to_vec(),
                extracted_text,
                tags,
                folder_id,
            },
            &state.file_storage,
        )
        .await?
        .map_err(unprocessable)?;

    Ok(Response::Created(article.into()))
}
