use std::{
    future::Future,
    sync::{Arc, LazyLock},
    time::Duration,
};

use axum::{
    Json,
    body::Bytes,
    response::{IntoResponse, Response},
};
use handlers::ApiError;
use http::{HeaderValue, StatusCode, header::RETRY_AFTER};
use mestier_core::{NormalizedImage, SURVEY_DECODE_LIMITS, normalize_image_with};
use serde::Serialize;
use tokio::sync::Semaphore;
use utoipa::ToSchema;

pub const MEMBER_CONCURRENT_NORMALIZATIONS: usize = 4;
pub const PUBLIC_CONCURRENT_NORMALIZATIONS: usize = 2;
pub const PERMIT_WAIT: Duration = Duration::from_secs(2);
const NORMALIZATION_TIMEOUT: Duration = Duration::from_secs(60);

static MEMBER_PERMITS: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(MEMBER_CONCURRENT_NORMALIZATIONS)));
static PUBLIC_PERMITS: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(PUBLIC_CONCURRENT_NORMALIZATIONS)));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Uploader {
    Member,
    Public,
}

impl Uploader {
    fn permits(self) -> &'static Arc<Semaphore> {
        match self {
            Self::Member => &MEMBER_PERMITS,
            Self::Public => &PUBLIC_PERMITS,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct UploadedSurveyPhoto {
    pub key: String,
}

#[derive(Debug)]
pub enum PhotoError {
    TooLarge(u64),
    Busy,
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
            Self::Busy => {
                let mut response = (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({
                        "code": "E_SERVICE_UNAVAILABLE",
                        "message": "too many photos are being processed, retry in a moment",
                        "status": 503,
                    })),
                )
                    .into_response();
                response
                    .headers_mut()
                    .insert(RETRY_AFTER, HeaderValue::from_static("2"));
                response
            }
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

pub(crate) async fn checked_then_normalized<C>(
    check: C,
    body: Bytes,
    uploader: Uploader,
) -> Result<NormalizedImage, PhotoError>
where
    C: Future<Output = Result<(), PhotoError>>,
{
    check.await?;
    normalized(body, uploader.permits(), PERMIT_WAIT).await
}

pub(crate) async fn normalized(
    body: Bytes,
    permits: &Arc<Semaphore>,
    wait: Duration,
) -> Result<NormalizedImage, PhotoError> {
    let permit = match tokio::time::timeout(wait, permits.clone().acquire_owned()).await {
        Ok(Ok(permit)) => permit,
        Ok(Err(_)) => return Err(ApiError::Internal.into()),
        Err(_) => return Err(PhotoError::Busy),
    };
    let task = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        normalize_image_with(&body, SURVEY_DECODE_LIMITS)
    });
    match tokio::time::timeout(NORMALIZATION_TIMEOUT, task).await {
        Ok(Ok(outcome)) => {
            outcome.map_err(|error| ApiError::UnprocessableEntity(error.to_string()).into())
        }
        Ok(Err(error)) => {
            tracing::error!(error = %error, "survey photo normalization task failed");
            Err(ApiError::Internal.into())
        }
        Err(_) => {
            tracing::warn!("survey photo normalization exceeded {NORMALIZATION_TIMEOUT:?}");
            Err(ApiError::Internal.into())
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

    async fn status_of(
        result: Result<NormalizedImage, PhotoError>,
    ) -> (StatusCode, serde_json::Value) {
        let Err(error) = result else {
            panic!("the photo must be refused");
        };
        let response = error.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap_or_default())
    }

    fn free() -> Arc<Semaphore> {
        Arc::new(Semaphore::new(1))
    }

    #[tokio::test]
    async fn a_file_that_is_not_an_image_is_unprocessable() {
        let refused = normalized(
            Bytes::from_static(b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n"),
            &free(),
            PERMIT_WAIT,
        )
        .await;

        assert_eq!(status_of(refused).await.0, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn a_gif_is_refused_on_its_content() {
        let refused = normalized(
            Bytes::from_static(b"GIF89a\x01\x00\x01\x00\x00\x00\x00;"),
            &free(),
            PERMIT_WAIT,
        )
        .await;

        assert!(refused.is_err());
    }

    #[tokio::test]
    async fn a_photo_waiting_too_long_for_a_permit_answers_503() {
        let exhausted = Arc::new(Semaphore::new(0));

        let refused = normalized(
            Bytes::from_static(b"GIF89a"),
            &exhausted,
            Duration::from_millis(20),
        )
        .await;
        let (status, json) = status_of(refused).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(json["code"], "E_SERVICE_UNAVAILABLE");
        assert_eq!(PERMIT_WAIT, Duration::from_secs(2));
    }

    #[tokio::test]
    async fn a_capped_response_is_refused_before_waiting_for_a_permit() {
        let held = PUBLIC_PERMITS
            .clone()
            .acquire_many_owned(PUBLIC_CONCURRENT_NORMALIZATIONS as u32)
            .await
            .unwrap();
        let capped = async {
            Err(PhotoError::Api(ApiError::Conflict(
                "response_photo_cap_reached".to_owned(),
            )))
        };

        let refused = tokio::time::timeout(
            Duration::from_millis(500),
            checked_then_normalized(capped, Bytes::from_static(b"GIF89a"), Uploader::Public),
        )
        .await
        .expect("a capped response never waits for a permit");
        drop(held);

        assert_eq!(status_of(refused).await.0, StatusCode::CONFLICT);
    }

    #[test]
    fn the_public_route_normalizes_two_photos_at_a_time_and_members_four() {
        assert_eq!(PUBLIC_CONCURRENT_NORMALIZATIONS, 2);
        assert_eq!(MEMBER_CONCURRENT_NORMALIZATIONS, 4);
    }
}
