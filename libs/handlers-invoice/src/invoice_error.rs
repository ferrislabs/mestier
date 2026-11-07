use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use handlers::ApiError;
use http::StatusCode;
use mestier_core::InvoiceError;
use serde_json::json;

#[derive(Debug)]
pub enum InvoiceFailure {
    Api(ApiError),
    Refused(InvoiceError),
}

impl std::fmt::Display for InvoiceFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::Refused(error) => error.fmt(f),
        }
    }
}

impl From<ApiError> for InvoiceFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<CoreError> for InvoiceFailure {
    fn from(error: CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<InvoiceError> for InvoiceFailure {
    fn from(error: InvoiceError) -> Self {
        Self::Refused(error)
    }
}

fn status_of(error: InvoiceError) -> StatusCode {
    if error.is_state_conflict() {
        StatusCode::CONFLICT
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

impl IntoResponse for InvoiceFailure {
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
    use mestier_core::InvoiceField;

    use super::*;

    async fn body_of(failure: InvoiceFailure) -> (StatusCode, serde_json::Value) {
        let response = failure.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn a_locked_invoice_answers_409_with_its_code() {
        let (status, body) = body_of(InvoiceError::Locked.into()).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "E_INVOICE_LOCKED");
        assert_eq!(body["details"]["field"], "status");
    }

    #[tokio::test]
    async fn an_already_invoiced_quote_answers_409_with_its_code() {
        let (status, body) = body_of(InvoiceError::QuoteAlreadyInvoiced.into()).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "E_QUOTE_ALREADY_INVOICED");
    }

    #[tokio::test]
    async fn a_payment_above_the_balance_answers_422_naming_the_field() {
        let (status, body) = body_of(InvoiceError::PaymentExceedsBalance.into()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_PAYMENT_EXCEEDS_BALANCE");
        assert_eq!(body["details"]["field"], "amount_cents");
    }

    #[tokio::test]
    async fn a_refused_rule_answers_422_naming_the_field() {
        let refusal = InvoiceError::invalid(InvoiceField::Reason, "required");
        let (status, body) = body_of(refusal.into()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_UNPROCESSABLE_ENTITY");
        assert_eq!(body["details"]["field"], "reason");
    }

    #[tokio::test]
    async fn an_unknown_invoice_stays_a_404() {
        let (status, body) = body_of(CoreError::NotFound.into()).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "E_NOT_FOUND");
    }
}
