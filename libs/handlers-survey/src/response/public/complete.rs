use axum::{
    extract::{State, rejection::PathRejection},
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::response::{
    RespondentResponseView,
    public::{PublicResponseCompletionPath, hidden, locate},
};

#[utoipa::path(
    post,
    path = "/api/v1/public/surveys/{slug}/responses/{resume_token}/complete",
    operation_id = "completePublicSurveyResponse",
    tag = super::super::super::TAG,
    description = "Completes a public response. Rate limited per client address (600 calls a minute).",
    params(
        ("slug" = String, Path, description = "The public slug of the survey"),
        ("resume_token" = String, Path, description = "The resume token returned when the response was started"),
    ),
    responses(
        (status = 200, description = "Response completed; survey.response_completed is emitted once", body = inline(DataEnvelope<RespondentResponseView>)),
        (status = 404, description = "No such survey or response"),
        (status = 409, description = "Closed, outside its campaign window, response cap reached, already complete or already answered (details.reason)"),
        (status = 422, description = "A required question on the path is unanswered"),
        (status = 429, description = "Too many requests"),
    ),
)]
pub async fn handler(
    path: Result<PublicResponseCompletionPath, PathRejection>,
    State(state): State<AppState>,
) -> Result<HttpResponse, ApiError> {
    let PublicResponseCompletionPath { slug, resume_token } =
        path.map_err(|_| ApiError::NotFound)?;
    let (slug, token) = locate(slug, resume_token)?;

    let response = state
        .usecase
        .complete_public_response(&slug, token)
        .await
        .map_err(hidden)?;

    Ok(Response::OK(RespondentResponseView::from(&response)).into_response())
}
