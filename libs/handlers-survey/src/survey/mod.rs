use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::AppState;
use mestier_core::{
    CampaignId, Closed, Draft, Metadata, Opened, OrganizationId, PublicSlug, Survey, SurveyGraph,
    SurveyId, SurveyRecord, Visibility,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod close;
pub mod create;
pub mod delete;
pub mod duplicate;
pub mod get_one;
pub mod list;
pub mod open;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
        .typed_post(open::handler)
        .typed_post(close::handler)
        .typed_post(duplicate::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surveys")]
pub struct SurveysPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surveys/{survey_id}")]
pub struct SurveyPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surveys/{survey_id}/open")]
pub struct OpenSurveyPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surveys/{survey_id}/close")]
pub struct CloseSurveyPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surveys/{survey_id}/duplicate")]
pub struct DuplicateSurveyPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SurveyStatus {
    Draft,
    Opened,
    Closed,
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct SurveyView<'a> {
    pub id: SurveyId,
    pub organization_id: OrganizationId,
    pub status: SurveyStatus,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub visibility: Visibility,
    pub allow_multiple_responses: bool,
    pub response_cap: Option<u32>,
    #[schema(value_type = BTreeMap<String, String>)]
    pub metadata: &'a Metadata,
    pub campaign_id: Option<CampaignId>,
    #[schema(value_type = Option<crate::schema::SurveyGraphSchema>)]
    pub graph: Option<&'a SurveyGraph>,
    #[schema(value_type = Option<String>)]
    pub public_slug: Option<&'a PublicSlug>,
    pub opened_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

macro_rules! survey_view {
    ($survey:expr, $status:expr, $public_slug:expr) => {
        SurveyView {
            id: $survey.id(),
            organization_id: $survey.organization_id(),
            status: $status,
            title: $survey.title(),
            description: $survey.description(),
            visibility: $survey.visibility(),
            allow_multiple_responses: $survey.allow_multiple_responses(),
            response_cap: $survey.response_cap().map(|cap| cap.get()),
            metadata: $survey.metadata(),
            campaign_id: $survey.campaign_id(),
            graph: $survey.graph(),
            public_slug: $public_slug,
            opened_at: $survey.opened_at(),
            closed_at: $survey.closed_at(),
        }
    };
}

impl<'a> SurveyView<'a> {
    pub fn draft(survey: &'a Survey<Draft>) -> Self {
        survey_view!(survey, SurveyStatus::Draft, None)
    }

    pub fn opened(survey: &'a Survey<Opened>) -> Self {
        survey_view!(survey, SurveyStatus::Opened, survey.public_slug())
    }

    pub fn closed(survey: &'a Survey<Closed>) -> Self {
        survey_view!(survey, SurveyStatus::Closed, survey.public_slug())
    }
}

impl<'a> From<&'a SurveyRecord> for SurveyView<'a> {
    fn from(record: &'a SurveyRecord) -> Self {
        match record {
            SurveyRecord::Draft(survey) => Self::draft(survey),
            SurveyRecord::Opened(survey) => Self::opened(survey),
            SurveyRecord::Closed(survey) => Self::closed(survey),
        }
    }
}
