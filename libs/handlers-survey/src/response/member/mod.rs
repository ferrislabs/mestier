use axum_extra::routing::TypedPath;
use mestier_core::{OrganizationId, ResponseId, SurveyId};
use serde::Deserialize;

pub mod complete;
pub mod open;
pub mod photo;
pub mod save;
pub mod start;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/member-surveys/{survey_id}")]
pub struct MemberSurveyPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/member-surveys/{survey_id}/responses")]
pub struct MemberSurveyResponsesPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/member-survey-responses/{response_id}")]
pub struct MemberResponsePath {
    pub organization_id: OrganizationId,
    pub response_id: ResponseId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/member-survey-responses/{response_id}/complete"
)]
pub struct MemberResponseCompletionPath {
    pub organization_id: OrganizationId,
    pub response_id: ResponseId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/member-survey-responses/{response_id}/photos"
)]
pub struct MemberResponsePhotosPath {
    pub organization_id: OrganizationId,
    pub response_id: ResponseId,
}
