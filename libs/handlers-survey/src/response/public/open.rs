use axum::{
    extract::{State, rejection::PathRejection},
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::response::{
    RespondentSurveyView,
    public::{PublicSurveyPath, hidden, slug},
};

#[utoipa::path(
    get,
    path = "/api/v1/public/surveys/{slug}",
    operation_id = "openPublicSurvey",
    tag = super::super::super::TAG,
    description = "Reads an opened public survey by its link, without an account. The slug is a secret: it is redacted from the logs. An unknown slug, a malformed slug, a draft, a members survey and a survey whose organization has not enabled surveys all answer the same 404. Rate limited per client address (600 calls a minute).",
    params(
        ("slug" = String, Path, description = "The public slug of the survey"),
    ),
    responses(
        (status = 200, description = "The opened public survey as its respondent sees it: no organization, metadata, settings nor campaign", body = inline(DataEnvelope<RespondentSurveyView>)),
        (status = 404, description = "No opened public survey behind this link"),
        (status = 409, description = "The survey is closed (details.reason = survey_closed)"),
        (status = 429, description = "Too many requests"),
    ),
)]
pub async fn handler(
    path: Result<PublicSurveyPath, PathRejection>,
    State(state): State<AppState>,
) -> Result<HttpResponse, ApiError> {
    let PublicSurveyPath { slug: value } = path.map_err(|_| ApiError::NotFound)?;
    let slug = slug(value)?;

    let survey = state
        .usecase
        .open_public_survey(&slug)
        .await
        .map_err(hidden)?;

    Ok(Response::OK(RespondentSurveyView::from(&survey)).into_response())
}
