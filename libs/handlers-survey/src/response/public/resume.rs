use axum::{
    extract::{State, rejection::PathRejection},
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::response::{
    RespondentResponseView,
    public::{PublicResponsePath, hidden, locate},
};

#[utoipa::path(
    get,
    path = "/api/v1/public/surveys/{slug}/responses/{resume_token}",
    operation_id = "resumePublicSurveyResponse",
    tag = super::super::super::TAG,
    description = "Reads back a public response in progress by its resume token. An unknown or malformed token answers the same 404 as an unknown survey. Rate limited per client address (600 calls a minute).",
    params(
        ("slug" = String, Path, description = "The public slug of the survey"),
        ("resume_token" = String, Path, description = "The resume token returned when the response was started"),
    ),
    responses(
        (status = 200, description = "The response in progress", body = inline(DataEnvelope<RespondentResponseView>)),
        (status = 404, description = "No such survey or response"),
        (status = 409, description = "The response is already complete or the survey is closed (details.reason)"),
        (status = 429, description = "Too many requests"),
    ),
)]
pub async fn handler(
    path: Result<PublicResponsePath, PathRejection>,
    State(state): State<AppState>,
) -> Result<HttpResponse, ApiError> {
    let PublicResponsePath { slug, resume_token } = path.map_err(|_| ApiError::NotFound)?;
    let (slug, token) = locate(slug, resume_token)?;

    let response = state
        .usecase
        .resume_public_response(&slug, token)
        .await
        .map_err(hidden)?;

    Ok(Response::OK(RespondentResponseView::from(&response)).into_response())
}
