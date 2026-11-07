use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use handlers::ApiError;
use http::StatusCode;
use mestier_core::QuoteError;
use serde_json::json;

#[derive(Debug)]
pub enum QuoteFailure {
    Api(ApiError),
    Refused(QuoteError),
}

impl std::fmt::Display for QuoteFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::Refused(error) => error.fmt(f),
        }
    }
}

impl From<ApiError> for QuoteFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<CoreError> for QuoteFailure {
    fn from(error: CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<QuoteError> for QuoteFailure {
    fn from(error: QuoteError) -> Self {
        Self::Refused(error)
    }
}

fn status_of(error: QuoteError) -> StatusCode {
    if error.is_state_conflict() {
        StatusCode::CONFLICT
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

impl IntoResponse for QuoteFailure {
    fn into_response(self) -> Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => {
                let status = status_of(error);
                (
                    status,
                    Json(json!({
                        "code": error.code(),
                        "message": error.to_string(),
                        "status": status.as_u16(),
                        "details": { "field": error.field().as_str() },
                    })),
                )
                    .into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use mestier_core::QuoteField;

    use super::*;

    async fn body_of(failure: QuoteFailure) -> (StatusCode, serde_json::Value) {
        let response = failure.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn a_locked_quote_answers_409_with_its_code() {
        let (status, body) = body_of(QuoteError::Locked.into()).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "E_QUOTE_LOCKED");
        assert_eq!(body["details"]["field"], "status");
    }

    #[tokio::test]
    async fn an_expired_quote_answers_409_with_its_code() {
        let (status, body) = body_of(QuoteError::Expired.into()).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "E_QUOTE_EXPIRED");
        assert_eq!(body["details"]["field"], "valid_until");
    }

    #[tokio::test]
    async fn answering_a_quote_that_was_not_sent_answers_409_with_its_code() {
        let (status, body) = body_of(QuoteError::NotSent.into()).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "E_QUOTE_NOT_SENT");
    }

    #[tokio::test]
    async fn a_refused_rule_answers_422_naming_the_field() {
        let refusal = QuoteError::invalid(QuoteField::DepositPercent, "out of range");
        let (status, body) = body_of(refusal.into()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_UNPROCESSABLE_ENTITY");
        assert_eq!(body["details"]["field"], "deposit_percent");
    }

    #[tokio::test]
    async fn an_unknown_quote_stays_a_404() {
        let (status, body) = body_of(CoreError::NotFound.into()).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "E_NOT_FOUND");
    }
}
