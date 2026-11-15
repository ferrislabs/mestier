use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::AppState;
use mestier_core::{
    CampaignId, Closed, Draft, Metadata, Opened, OrganizationId, PublicSlug, Survey, SurveyGraph,
    SurveyId, SurveyRecord, SurveyState, SurveySummary, Visibility,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod close;
pub mod create;
pub mod delete;
pub mod duplicate;
pub mod get_one;
pub mod graph_version;
pub mod list;
pub mod open;
pub mod reopen;
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
        .typed_post(reopen::handler)
        .typed_post(duplicate::handler)
        .typed_get(graph_version::handler)
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
#[typed_path("/api/v1/organizations/{organization_id}/surveys/{survey_id}/reopen")]
pub struct ReopenSurveyPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surveys/{survey_id}/duplicate")]
pub struct DuplicateSurveyPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/surveys/{survey_id}/graph-versions/{version}"
)]
pub struct SurveyGraphVersionPath {
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
    pub version: u32,
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
    #[schema(value_type = Vec<CampaignId>)]
    pub campaign_ids: &'a [CampaignId],
    #[schema(value_type = Option<crate::schema::SurveyGraphSchema>)]
    pub graph: Option<&'a SurveyGraph>,
    #[schema(minimum = 1)]
    pub graph_version: Option<u32>,
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
            campaign_ids: $survey.campaign_ids(),
            graph: $survey.graph(),
            graph_version: $survey.graph_version().map(|version| version.get()),
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

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct SurveySummaryView<'a> {
    pub id: SurveyId,
    pub title: &'a str,
    pub status: SurveyStatus,
}

impl<'a> From<&'a SurveySummary> for SurveySummaryView<'a> {
    fn from(summary: &'a SurveySummary) -> Self {
        Self {
            id: summary.id,
            title: &summary.title,
            status: match summary.state {
                SurveyState::Draft => SurveyStatus::Draft,
                SurveyState::Opened => SurveyStatus::Opened,
                SurveyState::Closed => SurveyStatus::Closed,
            },
        }
    }
}
