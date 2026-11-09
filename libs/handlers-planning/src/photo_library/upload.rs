use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use auth::Identity;
use axum::{
    Extension, Json,
    body::Bytes,
    extract::{Query, State},
    response::IntoResponse,
};
use handlers::{ApiError, AppState, Response, require_org_membership, resolve_actor};
use http::StatusCode;
use mestier_core::{NormalizedImage, ProjectId, UploadLibraryPhotoCommand, normalize_image};
use serde::Deserialize;
use tokio::sync::Semaphore;

use crate::photo_library::{PhotosPath, response::LibraryPhotoResponse, unprocessable};

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    pub project_id: Option<ProjectId>,
}

#[derive(Debug)]
pub enum UploadError {
    TooLarge(u64),
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
            Self::TooLarge(limit) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(serde_json::json!({
                    "code": "E_PAYLOAD_TOO_LARGE",
                    "message": format!("image exceeds the maximum upload size of {limit} bytes"),
                    "status": 413,
                })),
            )
                .into_response(),
        }
    }
}

const CONCURRENT_NORMALIZATIONS: usize = 4;
const NORMALIZATION_TIMEOUT: Duration = Duration::from_secs(60);

static NORMALIZATION_PERMITS: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(CONCURRENT_NORMALIZATIONS)));

async fn normalize_off_the_executor(bytes: Bytes) -> Result<NormalizedImage, ApiError> {
    let permit = NORMALIZATION_PERMITS
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| ApiError::Internal)?;
    let task = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        normalize_image(&bytes)
    });
    match tokio::time::timeout(NORMALIZATION_TIMEOUT, task).await {
        Ok(Ok(outcome)) => outcome.map_err(unprocessable),
        Ok(Err(error)) => {
            tracing::error!(error = %error, "image normalization task failed");
            Err(ApiError::Internal)
        }
        Err(_) => {
            tracing::warn!("image normalization exceeded {NORMALIZATION_TIMEOUT:?}");
            Err(ApiError::Internal)
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/photo-library/photos",
    operation_id = "uploadLibraryPhoto",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = Option<mestier_core::ProjectId>, Query, description = "Project the photo comes from"),
    ),
    request_body(content = Vec<u8>, content_type = "image/jpeg"),
    responses(
        (status = 201, description = "The photo, normalized and stored", body = inline(handlers::DataEnvelope<LibraryPhotoResponse>)),
        (status = 200, description = "The organization already holds this image: the existing photo", body = inline(handlers::DataEnvelope<LibraryPhotoResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
        (status = 413, description = "The file exceeds the maximum upload size"),
        (status = 422, description = "Not a JPEG, PNG or WebP image, or empty, or corrupt"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PhotosPath { organization_id }: PhotosPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<UploadQuery>,
    body: Bytes,
) -> Result<Response<LibraryPhotoResponse>, UploadError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let limit = state.args.file_storage.max_upload_bytes;
    if body.len() as u64 > limit {
        return Err(UploadError::TooLarge(limit));
    }

    let image = normalize_off_the_executor(body).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let uploaded = state
        .usecase
        .acting_as(user_id)
        .upload_library_photo(
            UploadLibraryPhotoCommand {
                actor,
                organization_id,
                project_id: query.project_id,
                image,
            },
            &state.file_storage,
        )
        .await?;

    Ok(if uploaded.created {
        Response::Created(uploaded.photo.into())
    } else {
        Response::OK(uploaded.photo.into())
    })
}
