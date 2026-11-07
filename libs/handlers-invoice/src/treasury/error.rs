use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use handlers::ApiError;
use http::StatusCode;
use mestier_core::{ExpenseError, LedgerError};
use serde_json::json;

fn refusal(status: StatusCode, code: &str, message: String, field: &str) -> Response {
    (
        status,
        Json(json!({
            "code": code,
            "message": message,
            "status": status.as_u16(),
            "details": { "field": field },
        })),
    )
        .into_response()
}

#[derive(Debug)]
pub enum ExpenseFailure {
    Api(ApiError),
    Refused(ExpenseError),
}

impl std::fmt::Display for ExpenseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::Refused(error) => error.fmt(f),
        }
    }
}

impl From<ApiError> for ExpenseFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<CoreError> for ExpenseFailure {
    fn from(error: CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<ExpenseError> for ExpenseFailure {
    fn from(error: ExpenseError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for ExpenseFailure {
    fn into_response(self) -> Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => {
                let status = if error.is_conflict() {
                    StatusCode::CONFLICT
                } else {
                    StatusCode::UNPROCESSABLE_ENTITY
                };
                refusal(
                    status,
                    error.code(),
                    error.to_string(),
                    error.field().as_str(),
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum LedgerFailure {
    Api(ApiError),
    Refused(LedgerError),
    BadMonth,
}

impl std::fmt::Display for LedgerFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::Refused(error) => error.fmt(f),
            Self::BadMonth => f.write_str("month: must be formatted YYYY-MM"),
        }
    }
}

impl From<ApiError> for LedgerFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<CoreError> for LedgerFailure {
    fn from(error: CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<LedgerError> for LedgerFailure {
    fn from(error: LedgerError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for LedgerFailure {
    fn into_response(self) -> Response {
        let status = StatusCode::UNPROCESSABLE_ENTITY;
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => refusal(
                status,
                "E_UNPROCESSABLE_ENTITY",
                error.to_string(),
                error.field(),
            ),
            Self::BadMonth => refusal(
                status,
                "E_UNPROCESSABLE_ENTITY",
                Self::BadMonth.to_string(),
                "month",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    async fn body_of(response: Response) -> (StatusCode, serde_json::Value) {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn a_rule_refused_on_a_field_answers_422_naming_it() {
        let (status, body) =
            body_of(ExpenseFailure::from(ExpenseError::AmountNotPositive).into_response()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_UNPROCESSABLE_ENTITY");
        assert_eq!(body["details"]["field"], "amount_ht_cents");
    }

    #[tokio::test]
    async fn a_state_conflict_answers_409_with_its_code() {
        let (status, body) =
            body_of(ExpenseFailure::from(ExpenseError::NotPayable).into_response()).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "E_EXPENSE_NOT_PAYABLE");
        assert_eq!(body["details"]["field"], "status");
    }

    #[tokio::test]
    async fn a_category_in_use_answers_409() {
        let (status, body) =
            body_of(ExpenseFailure::from(ExpenseError::CategoryInUse).into_response()).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "E_EXPENSE_CATEGORY_IN_USE");
    }

    #[tokio::test]
    async fn a_bad_month_answers_422_naming_the_month() {
        let (status, body) = body_of(LedgerFailure::BadMonth.into_response()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["details"]["field"], "month");
    }

    #[tokio::test]
    async fn a_receipt_without_a_positive_amount_answers_422() {
        let (status, body) =
            body_of(LedgerFailure::from(LedgerError::AmountNotPositive).into_response()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["details"]["field"], "amount_cents");
    }

    #[tokio::test]
    async fn an_unknown_id_stays_a_404() {
        let (status, _) = body_of(ExpenseFailure::from(CoreError::NotFound).into_response()).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
