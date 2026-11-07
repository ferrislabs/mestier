use axum::{
    Json,
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use mestier_core::CollectionError;

use super::{REQUEST_LIMIT_BYTES, dto::CollectionRequest};
use crate::error::AutomationError;

pub struct CollectionBody(pub CollectionRequest);

impl<S: Send + Sync> FromRequest<S> for CollectionBody {
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<CollectionRequest>::from_request(request, state).await {
            Ok(Json(body)) => Ok(Self(body)),
            Err(rejection) if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => {
                Err(AutomationError::from(CollectionError::RequestTooLarge {
                    limit: REQUEST_LIMIT_BYTES,
                })
                .into_response())
            }
            Err(rejection) => Err(rejection.into_response()),
        }
    }
}
