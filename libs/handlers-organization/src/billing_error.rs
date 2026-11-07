use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use handlers::ApiError;
use http::StatusCode;
use mestier_core::{BillingSettingsError, NumberingError};
use serde_json::json;

#[derive(Debug)]
pub enum BillingError {
    Api(ApiError),
    InvalidSetting(BillingSettingsError),
    NumberNotForward(NumberingError),
}

impl std::fmt::Display for BillingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::InvalidSetting(error) => error.fmt(f),
            Self::NumberNotForward(error) => error.fmt(f),
        }
    }
}

impl From<ApiError> for BillingError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<CoreError> for BillingError {
    fn from(error: CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<BillingSettingsError> for BillingError {
    fn from(error: BillingSettingsError) -> Self {
        Self::InvalidSetting(error)
    }
}

impl From<NumberingError> for BillingError {
    fn from(error: NumberingError) -> Self {
        Self::NumberNotForward(error)
    }
}

impl IntoResponse for BillingError {
    fn into_response(self) -> Response {
        let status = StatusCode::UNPROCESSABLE_ENTITY;
        match self {
            Self::Api(error) => error.into_response(),
            Self::InvalidSetting(error) => (
                status,
                Json(json!({
                    "code": "E_UNPROCESSABLE_ENTITY",
                    "message": error.to_string(),
                    "status": status.as_u16(),
                    "details": { "field": error.field.as_str() },
                })),
            )
                .into_response(),
            Self::NumberNotForward(NumberingError::NotForward { last_assigned }) => (
                status,
                Json(json!({
                    "code": "E_NUMBER_NOT_FORWARD",
                    "message": NumberingError::NotForward { last_assigned }.to_string(),
                    "status": status.as_u16(),
                    "details": {
                        "field": "next_number",
                        "last_assigned_number": last_assigned,
                    },
                })),
            )
                .into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use mestier_core::BillingSettingsField;

    use super::*;

    async fn body_of(error: BillingError) -> (StatusCode, serde_json::Value) {
        let response = error.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn an_invalid_setting_answers_422_naming_the_field() {
        let (status, body) = body_of(BillingError::InvalidSetting(BillingSettingsError {
            field: BillingSettingsField::PaymentTermsDays,
            rule: "must be between 0 and 60 days",
        }))
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_UNPROCESSABLE_ENTITY");
        assert_eq!(body["details"]["field"], "payment_terms_days");
    }

    #[tokio::test]
    async fn a_number_not_forward_answers_422_with_its_own_code() {
        let (status, body) = body_of(BillingError::NumberNotForward(NumberingError::NotForward {
            last_assigned: 10,
        }))
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_NUMBER_NOT_FORWARD");
        assert_eq!(body["details"]["last_assigned_number"], 10);
    }

    #[tokio::test]
    async fn a_missing_organization_stays_a_404() {
        let (status, body) = body_of(CoreError::NotFound.into()).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "E_NOT_FOUND");
    }
}
