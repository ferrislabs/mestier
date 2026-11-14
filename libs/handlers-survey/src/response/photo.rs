use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use axum::{
    Json,
    body::Bytes,
    response::{IntoResponse, Response},
};
use handlers::ApiError;
use http::StatusCode;
use mestier_core::{NormalizedImage, normalize_image};
use serde::Serialize;
use tokio::sync::Semaphore;
use utoipa::ToSchema;

const CONCURRENT_NORMALIZATIONS: usize = 4;
const NORMALIZATION_TIMEOUT: Duration = Duration::from_secs(60);

static NORMALIZATION_PERMITS: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(CONCURRENT_NORMALIZATIONS)));

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct UploadedSurveyPhoto {
    pub key: String,
}

#[derive(Debug)]
pub enum PhotoError {
    TooLarge(u64),
    Api(ApiError),
}

impl From<ApiError> for PhotoError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<common::CoreError> for PhotoError {
    fn from(error: common::CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl IntoResponse for PhotoError {
    fn into_response(self) -> Response {
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

pub(crate) fn body_limit(max_upload_bytes: u64) -> usize {
    usize::try_from(max_upload_bytes)
        .unwrap_or(usize::MAX)
        .saturating_add(1)
}

pub(crate) fn within_size(body: &Bytes, max_upload_bytes: u64) -> Result<(), PhotoError> {
    if u64::try_from(body.len()).unwrap_or(u64::MAX) > max_upload_bytes {
        Err(PhotoError::TooLarge(max_upload_bytes))
    } else {
        Ok(())
    }
}

pub(crate) async fn normalized(body: Bytes) -> Result<NormalizedImage, ApiError> {
    let permit = NORMALIZATION_PERMITS
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| ApiError::Internal)?;
    let task = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        normalize_image(&body)
    });
    match tokio::time::timeout(NORMALIZATION_TIMEOUT, task).await {
        Ok(Ok(outcome)) => {
            outcome.map_err(|error| ApiError::UnprocessableEntity(error.to_string()))
        }
        Ok(Err(error)) => {
            tracing::error!(error = %error, "survey photo normalization task failed");
            Err(ApiError::Internal)
        }
        Err(_) => {
            tracing::warn!("survey photo normalization exceeded {NORMALIZATION_TIMEOUT:?}");
            Err(ApiError::Internal)
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    #[test]
    fn a_body_one_byte_above_the_limit_is_too_large() {
        assert!(within_size(&Bytes::from(vec![0; 10]), 10).is_ok());
        assert!(matches!(
            within_size(&Bytes::from(vec![0; 11]), 10),
            Err(PhotoError::TooLarge(10))
        ));
        assert_eq!(body_limit(10), 11);
        assert_eq!(body_limit(u64::MAX), usize::MAX);
    }

    #[tokio::test]
    async fn a_too_large_photo_answers_413() {
        let response = PhotoError::TooLarge(10).into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(json["code"], "E_PAYLOAD_TOO_LARGE");
    }

    #[tokio::test]
    async fn a_file_that_is_not_an_image_is_unprocessable() {
        let refused = normalized(Bytes::from_static(b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n")).await;
        let Err(error) = refused else {
            panic!("a pdf must be refused");
        };

        assert_eq!(
            error.into_response().status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[tokio::test]
    async fn a_gif_is_refused_on_its_content() {
        let refused = normalized(Bytes::from_static(b"GIF89a\x01\x00\x01\x00\x00\x00\x00;")).await;

        assert!(refused.is_err());
    }
}
