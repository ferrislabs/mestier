use axum::{Router, extract::DefaultBodyLimit, routing::post};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::AppState;
use mestier_core::{
    Answers, Completed, InProgress, MemberId, OrganizationId, RespondentSurvey, ResponseId,
    ResponseRecord, SurveyGraph, SurveyId, SurveyResponse, SurveyResponsePhase, Visibility,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod delete;
pub mod get_one;
pub mod list;
pub mod member;
pub mod photo;
pub mod public;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_get(get_one::handler)
        .typed_delete(delete::handler)
        .typed_get(member::open::handler)
        .typed_post(member::start::handler)
        .typed_put(member::save::handler)
        .typed_post(member::complete::handler)
        .route(
            member::MemberResponsePhotosPath::PATH,
            post(member::photo::handler).layer(DefaultBodyLimit::max(photo::body_limit(
                state.args.file_storage.max_upload_bytes,
            ))),
        )
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surveys/{survey_id}/responses")]
pub struct SurveyResponsesPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/survey-responses/{response_id}")]
pub struct SurveyResponsePath {
    pub organization_id: OrganizationId,
    pub response_id: ResponseId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResponseStatus {
    InProgress,
    Completed,
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct ResponseView<'a> {
    pub id: ResponseId,
    pub survey_id: SurveyId,
    pub member_id: Option<MemberId>,
    pub status: ResponseStatus,
    #[schema(value_type = BTreeMap<String, crate::schema::SurveyAnswerSchema>)]
    pub answers: &'a Answers,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    #[schema(minimum = 1)]
    pub graph_version: Option<u32>,
}

impl<'a> ResponseView<'a> {
    fn of<S: SurveyResponsePhase>(
        response: &'a SurveyResponse<S>,
        status: ResponseStatus,
        graph_version: Option<u32>,
    ) -> Self {
        Self {
            id: response.id(),
            survey_id: response.survey_id(),
            member_id: response.member_id(),
            status,
            answers: response.answers(),
            started_at: response.started_at(),
            completed_at: response.completed_at(),
            graph_version,
        }
    }
}

impl<'a> From<&'a ResponseRecord> for ResponseView<'a> {
    fn from(record: &'a ResponseRecord) -> Self {
        match record {
            ResponseRecord::InProgress(response) => {
                Self::of(response, ResponseStatus::InProgress, None)
            }
            ResponseRecord::Completed(response) => Self::of(
                response,
                ResponseStatus::Completed,
                Some(response.graph_version().get()),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct RespondentResponseView<'a> {
    pub id: ResponseId,
    pub status: ResponseStatus,
    #[schema(value_type = BTreeMap<String, crate::schema::SurveyAnswerSchema>)]
    pub answers: &'a Answers,
}

impl<'a> From<&'a SurveyResponse<InProgress>> for RespondentResponseView<'a> {
    fn from(response: &'a SurveyResponse<InProgress>) -> Self {
        Self {
            id: response.id(),
            status: ResponseStatus::InProgress,
            answers: response.answers(),
        }
    }
}

impl<'a> From<&'a SurveyResponse<Completed>> for RespondentResponseView<'a> {
    fn from(response: &'a SurveyResponse<Completed>) -> Self {
        Self {
            id: response.id(),
            status: ResponseStatus::Completed,
            answers: response.answers(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct RespondentSurveyView<'a> {
    pub id: SurveyId,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub visibility: Visibility,
    #[schema(value_type = crate::schema::SurveyGraphSchema)]
    pub graph: &'a SurveyGraph,
}

impl<'a> From<&'a RespondentSurvey> for RespondentSurveyView<'a> {
    fn from(survey: &'a RespondentSurvey) -> Self {
        Self {
            id: survey.id(),
            title: survey.title(),
            description: survey.description(),
            visibility: survey.visibility(),
            graph: survey.graph(),
        }
    }
}
