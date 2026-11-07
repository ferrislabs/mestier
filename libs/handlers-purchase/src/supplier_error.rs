use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use handlers::ApiError;
use http::StatusCode;
use mestier_core::SupplierError;
use serde_json::json;

#[derive(Debug)]
pub enum SupplierFailure {
    Api(ApiError),
    Refused(SupplierError),
}

impl std::fmt::Display for SupplierFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::Refused(error) => error.fmt(f),
        }
    }
}

impl From<ApiError> for SupplierFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<CoreError> for SupplierFailure {
    fn from(error: CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<SupplierError> for SupplierFailure {
    fn from(error: SupplierError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for SupplierFailure {
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
    use mestier_core::SupplierField;

    use super::*;

    #[tokio::test]
    async fn a_refused_rule_answers_422_naming_the_field() {
        let response = SupplierFailure::from(SupplierError {
            field: SupplierField::RegistrationNumber,
            rule: "must hold 9 or 14 digits",
        })
        .into_response();
        let status = response.status();
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_UNPROCESSABLE_ENTITY");
        assert_eq!(body["details"]["field"], "registration_number");
    }

    #[tokio::test]
    async fn an_unknown_id_stays_a_404() {
        let response = SupplierFailure::from(CoreError::NotFound).into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
