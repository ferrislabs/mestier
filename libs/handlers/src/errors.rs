use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use http::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MiddlewareError {
    #[error("missing authentication header")]
    MissingAuthHeader,
    #[error("invalid authentication header")]
    InvalidAuthHeader,
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),
}

impl IntoResponse for MiddlewareError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            MiddlewareError::MissingAuthHeader => (
                StatusCode::UNAUTHORIZED,
                "E_MISSING_AUTH_HEADER",
                "Authorization header is missing",
            ),
            MiddlewareError::InvalidAuthHeader => (
                StatusCode::UNAUTHORIZED,
                "E_INVALID_AUTH_HEADER",
                "Invalid authorization header",
            ),
            MiddlewareError::AuthenticationFailed(_) => (
                StatusCode::UNAUTHORIZED,
                "E_AUTHENTICATION_FAILED",
                "Authentication failed",
            ),
        };

        (
            status,
            Json(ErrorBody {
                code,
                message: message.to_string(),
                status: status.as_u16(),
                details: None,
            }),
        )
            .into_response()
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    // 400
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    // 401
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Token not found")]
    TokenNotFound,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    // 403
    #[error("Forbidden")]
    Forbidden,

    #[error("Insufficient permissions: required scope `{0}`")]
    InsufficientScope(String),

    // 404
    #[error("Resource not found")]
    NotFound,

    #[error("{0} with id `{1}` not found")]
    ResourceNotFound(&'static str, String),

    // 409
    #[error("Conflict: {0}")]
    Conflict(String),

    // 422
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    // 429
    #[error("Too many requests")]
    TooManyRequests,

    // 500
    #[error("Internal server error")]
    Internal,

    #[error("Database error")]
    Database,

    #[error("External service error: {0}")]
    ExternalService(String),
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ApiError {
    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized | Self::TokenNotFound | Self::TokenExpired | Self::InvalidToken => {
                StatusCode::UNAUTHORIZED
            }
            Self::Forbidden | Self::InsufficientScope(_) => StatusCode::FORBIDDEN,
            Self::NotFound | Self::ResourceNotFound(_, _) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal | Self::Database | Self::ExternalService(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "E_BAD_REQUEST",
            Self::Validation(_) => "E_VALIDATION",
            Self::Unauthorized => "E_UNAUTHORIZED",
            Self::TokenNotFound => "E_TOKEN_NOT_FOUND",
            Self::TokenExpired => "E_TOKEN_EXPIRED",
            Self::InvalidToken => "E_INVALID_TOKEN",
            Self::Forbidden => "E_FORBIDDEN",
            Self::InsufficientScope(_) => "E_INSUFFICIENT_SCOPE",
            Self::NotFound => "E_NOT_FOUND",
            Self::ResourceNotFound(_, _) => "E_RESOURCE_NOT_FOUND",
            Self::Conflict(_) => "E_CONFLICT",
            Self::UnprocessableEntity(_) => "E_UNPROCESSABLE_ENTITY",
            Self::TooManyRequests => "E_TOO_MANY_REQUESTS",
            Self::Internal => "E_INTERNAL",
            Self::Database => "E_DATABASE",
            Self::ExternalService(_) => "E_EXTERNAL_SERVICE",
        }
    }
}

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::NotFound => Self::NotFound,
            CoreError::Conflict(msg) => Self::Conflict(msg),
            CoreError::Forbidden { .. } => Self::Forbidden,
            CoreError::Database(msg) => {
                tracing::error!(error = %msg, "core database error");
                Self::Database
            }
            CoreError::Internal(msg) => {
                tracing::error!(error = %msg, "core internal error");
                Self::Internal
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorBody {
            code: self.code(),
            message: self.to_string(),
            status: status.as_u16(),
            details: None,
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn parse_response(err: ApiError) -> (StatusCode, serde_json::Value) {
        let response = err.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        (status, json)
    }

    macro_rules! assert_error {
        ($err:expr, $status:expr, $code:expr) => {{
            let (status, json) = parse_response($err).await;
            assert_eq!(status, $status);
            assert_eq!(json["code"], $code);
            assert_eq!(json["status"], $status.as_u16());
            assert!(json["message"].is_string());
            assert!(json.get("details").is_none());
        }};
    }

    #[tokio::test]
    async fn test_400_bad_request() {
        assert_error!(
            ApiError::BadRequest("invalid field".into()),
            StatusCode::BAD_REQUEST,
            "E_BAD_REQUEST"
        );
    }

    #[tokio::test]
    async fn test_400_validation() {
        assert_error!(
            ApiError::Validation("email is required".into()),
            StatusCode::BAD_REQUEST,
            "E_VALIDATION"
        );
    }

    #[tokio::test]
    async fn test_401_unauthorized() {
        assert_error!(
            ApiError::Unauthorized,
            StatusCode::UNAUTHORIZED,
            "E_UNAUTHORIZED"
        );
    }

    #[tokio::test]
    async fn test_401_token_not_found() {
        assert_error!(
            ApiError::TokenNotFound,
            StatusCode::UNAUTHORIZED,
            "E_TOKEN_NOT_FOUND"
        );
    }

    #[tokio::test]
    async fn test_401_token_expired() {
        assert_error!(
            ApiError::TokenExpired,
            StatusCode::UNAUTHORIZED,
            "E_TOKEN_EXPIRED"
        );
    }

    #[tokio::test]
    async fn test_401_invalid_token() {
        assert_error!(
            ApiError::InvalidToken,
            StatusCode::UNAUTHORIZED,
            "E_INVALID_TOKEN"
        );
    }

    #[tokio::test]
    async fn test_403_forbidden() {
        assert_error!(ApiError::Forbidden, StatusCode::FORBIDDEN, "E_FORBIDDEN");
    }

    #[tokio::test]
    async fn test_403_insufficient_scope() {
        assert_error!(
            ApiError::InsufficientScope("admin:write".into()),
            StatusCode::FORBIDDEN,
            "E_INSUFFICIENT_SCOPE"
        );
    }

    #[tokio::test]
    async fn test_404_not_found() {
        assert_error!(ApiError::NotFound, StatusCode::NOT_FOUND, "E_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_404_resource_not_found() {
        let (status, json) =
            parse_response(ApiError::ResourceNotFound("Customer", "42".into())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["code"], "E_RESOURCE_NOT_FOUND");
        assert_eq!(json["status"], 404);
        assert!(json["message"].as_str().unwrap().contains("Customer"));
        assert!(json["message"].as_str().unwrap().contains("42"));
    }

    #[tokio::test]
    async fn test_409_conflict() {
        assert_error!(
            ApiError::Conflict("email already exists".into()),
            StatusCode::CONFLICT,
            "E_CONFLICT"
        );
    }

    #[tokio::test]
    async fn test_422_unprocessable_entity() {
        assert_error!(
            ApiError::UnprocessableEntity("invalid state transition".into()),
            StatusCode::UNPROCESSABLE_ENTITY,
            "E_UNPROCESSABLE_ENTITY"
        );
    }

    #[tokio::test]
    async fn test_429_too_many_requests() {
        assert_error!(
            ApiError::TooManyRequests,
            StatusCode::TOO_MANY_REQUESTS,
            "E_TOO_MANY_REQUESTS"
        );
    }

    #[tokio::test]
    async fn test_500_internal() {
        assert_error!(
            ApiError::Internal,
            StatusCode::INTERNAL_SERVER_ERROR,
            "E_INTERNAL"
        );
    }

    #[tokio::test]
    async fn test_500_database() {
        assert_error!(
            ApiError::Database,
            StatusCode::INTERNAL_SERVER_ERROR,
            "E_DATABASE"
        );
    }

    #[tokio::test]
    async fn test_500_external_service() {
        assert_error!(
            ApiError::ExternalService("payment gateway timeout".into()),
            StatusCode::INTERNAL_SERVER_ERROR,
            "E_EXTERNAL_SERVICE"
        );
    }

    #[tokio::test]
    async fn from_core_error_not_found_maps_to_404() {
        let api: ApiError = CoreError::NotFound.into();
        assert_error!(api, StatusCode::NOT_FOUND, "E_NOT_FOUND");
    }

    #[tokio::test]
    async fn from_core_error_conflict_preserves_message() {
        let api: ApiError = CoreError::Conflict("slug taken".into()).into();
        let (status, json) = parse_response(api).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json["code"], "E_CONFLICT");
        assert_eq!(json["status"], 409);
        assert!(json["message"].as_str().unwrap().contains("slug taken"));
    }

    #[tokio::test]
    async fn from_core_error_database_maps_to_500_database() {
        let api: ApiError = CoreError::Database("connection lost".into()).into();
        assert_error!(api, StatusCode::INTERNAL_SERVER_ERROR, "E_DATABASE");
    }

    #[tokio::test]
    async fn from_core_error_internal_maps_to_500_internal() {
        let api: ApiError = CoreError::Internal("boom".into()).into();
        assert_error!(api, StatusCode::INTERNAL_SERVER_ERROR, "E_INTERNAL");
    }

    #[tokio::test]
    async fn from_core_error_redacts_internal_message() {
        // Internal/Database details must not leak through the public message.
        let api: ApiError = CoreError::Database("password=hunter2".into()).into();
        let (_, json) = parse_response(api).await;
        let message = json["message"].as_str().unwrap();
        assert!(!message.contains("hunter2"));
    }

    #[tokio::test]
    async fn test_response_has_no_details_by_default() {
        let (_, json) = parse_response(ApiError::Internal).await;
        assert!(json.get("details").is_none());
    }
}
