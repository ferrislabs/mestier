use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use handlers::ApiError;
use http::StatusCode;
use mestier_core::CustomerError;
use serde_json::json;

#[derive(Debug)]
pub enum CustomerFailure {
    Api(ApiError),
    Refused(CustomerError),
}

impl std::fmt::Display for CustomerFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::Refused(error) => error.fmt(f),
        }
    }
}

impl From<ApiError> for CustomerFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<CoreError> for CustomerFailure {
    fn from(error: CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<CustomerError> for CustomerFailure {
    fn from(error: CustomerError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for CustomerFailure {
    fn into_response(self) -> Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => {
                let status = StatusCode::UNPROCESSABLE_ENTITY;
                (
                    status,
                    Json(json!({
                        "code": "E_UNPROCESSABLE_ENTITY",
                        "message": error.to_string(),
                        "status": status.as_u16(),
                        "details": { "field": error.field.as_str() },
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
    use mestier_core::CustomerField;

    use super::*;

    async fn body_of(failure: CustomerFailure) -> (StatusCode, serde_json::Value) {
        let response = failure.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn a_refused_field_answers_422_naming_it() {
        let (status, body) = body_of(CustomerFailure::Refused(CustomerError {
            field: CustomerField::PaymentTermsDays,
            rule: "must be between 0 and 60 days",
        }))
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_UNPROCESSABLE_ENTITY");
        assert_eq!(body["details"]["field"], "payment_terms_days");
    }

    #[tokio::test]
    async fn a_customer_of_another_organization_stays_a_404() {
        let (status, body) = body_of(CoreError::NotFound.into()).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "E_NOT_FOUND");
    }
}
