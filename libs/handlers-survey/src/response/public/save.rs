use axum::{
    Json,
    extract::{State, rejection::PathRejection},
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    request::answers,
    response::{
        RespondentResponseView,
        member::save::SaveSurveyAnswersRequest,
        public::{PublicResponsePath, hidden, locate},
    },
};

#[utoipa::path(
    put,
    path = "/api/v1/public/surveys/{slug}/responses/{resume_token}",
    operation_id = "savePublicSurveyResponse",
    tag = super::super::super::TAG,
    description = "Saves every answer given so far to a public response in progress. A photo answer lists keys returned by the photo upload of this same response. Rate limited per client address (600 calls a minute).",
    params(
        ("slug" = String, Path, description = "The public slug of the survey"),
        ("resume_token" = String, Path, description = "The resume token returned when the response was started"),
    ),
    request_body(
        content = SaveSurveyAnswersRequest,
        description = "Every answer given so far, keyed by question id, in the domain form of an answer. Answers off the path and blank answers are dropped.",
    ),
    responses(
        (status = 200, description = "Answers saved; survey.response_submitted is emitted", body = inline(DataEnvelope<RespondentResponseView>)),
        (status = 404, description = "No such survey or response"),
        (status = 409, description = "Closed, outside its campaign window, or already complete (details.reason)"),
        (status = 422, description = "An answer names an unknown question or option, does not match its kind, is an invalid email, phone or date, or names a photo not uploaded for this response"),
        (status = 429, description = "Too many requests"),
    ),
)]
pub async fn handler(
    path: Result<PublicResponsePath, PathRejection>,
    State(state): State<AppState>,
    Json(payload): Json<SaveSurveyAnswersRequest>,
) -> Result<HttpResponse, ApiError> {
    let PublicResponsePath { slug, resume_token } = path.map_err(|_| ApiError::NotFound)?;
    let (slug, token) = locate(slug, resume_token)?;
    let answers = answers(payload.answers)?;

    let response = state
        .usecase
        .save_public_response(&slug, token, answers)
        .await
        .map_err(hidden)?;

    Ok(Response::OK(RespondentResponseView::from(&response)).into_response())
}
