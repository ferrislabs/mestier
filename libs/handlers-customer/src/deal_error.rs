use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use handlers::ApiError;
use http::StatusCode;
use mestier_core::DealError;
use serde_json::json;

#[derive(Debug)]
pub enum DealFailure {
    Api(ApiError),
    Refused(DealError),
}

impl std::fmt::Display for DealFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::Refused(error) => error.fmt(f),
        }
    }
}

impl From<ApiError> for DealFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<CoreError> for DealFailure {
    fn from(error: CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<DealError> for DealFailure {
    fn from(error: DealError) -> Self {
        Self::Refused(error)
    }
}

fn status_of(error: DealError) -> StatusCode {
    match error {
        DealError::HasQuote => StatusCode::CONFLICT,
        DealError::TitleRequired
        | DealError::AmountNegative
        | DealError::OwnerNotMember
        | DealError::LostReasonRequired => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

impl IntoResponse for DealFailure {
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

    use super::*;

    async fn body_of(failure: DealFailure) -> (StatusCode, serde_json::Value) {
        let response = failure.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn a_lost_deal_without_reason_answers_422_with_its_code() {
        let (status, body) = body_of(DealError::LostReasonRequired.into()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_DEAL_LOST_REASON_REQUIRED");
        assert_eq!(body["details"]["field"], "lost_reason");
    }

    #[tokio::test]
    async fn deleting_a_deal_with_a_quote_answers_409_with_its_code() {
        let (status, body) = body_of(DealError::HasQuote.into()).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "E_DEAL_HAS_QUOTE");
    }

    #[tokio::test]
    async fn an_empty_title_answers_422_naming_the_field() {
        let (status, body) = body_of(DealError::TitleRequired.into()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_UNPROCESSABLE_ENTITY");
        assert_eq!(body["details"]["field"], "title");
    }

    #[tokio::test]
    async fn an_unknown_customer_stays_a_404() {
        let (status, body) = body_of(CoreError::NotFound.into()).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "E_NOT_FOUND");
    }
}
