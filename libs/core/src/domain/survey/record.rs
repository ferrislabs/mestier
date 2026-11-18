use super::{
    CampaignId, Closed, Completed, Draft, Graph, GraphVersion, InProgress, Opened, Response,
    ResponseId, Survey, SurveyId, Visibility,
};
use crate::OrganizationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurveyState {
    Draft,
    Opened,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurveySummary {
    pub id: SurveyId,
    pub title: String,
    pub state: SurveyState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurveyRecord {
    Draft(Survey<Draft>),
    Opened(Survey<Opened>),
    Closed(Survey<Closed>),
}

impl SurveyRecord {
    pub fn id(&self) -> SurveyId {
        match self {
            Self::Draft(survey) => survey.id(),
            Self::Opened(survey) => survey.id(),
            Self::Closed(survey) => survey.id(),
        }
    }

    pub fn organization_id(&self) -> OrganizationId {
        match self {
            Self::Draft(survey) => survey.organization_id(),
            Self::Opened(survey) => survey.organization_id(),
            Self::Closed(survey) => survey.organization_id(),
        }
    }

    pub fn visibility(&self) -> Visibility {
        match self {
            Self::Draft(survey) => survey.visibility(),
            Self::Opened(survey) => survey.visibility(),
            Self::Closed(survey) => survey.visibility(),
        }
    }

    pub fn campaign_ids(&self) -> &[CampaignId] {
        match self {
            Self::Draft(survey) => survey.campaign_ids(),
            Self::Opened(survey) => survey.campaign_ids(),
            Self::Closed(survey) => survey.campaign_ids(),
        }
    }

    pub fn graph(&self) -> Option<&Graph> {
        match self {
            Self::Draft(survey) => survey.graph(),
            Self::Opened(survey) => survey.graph(),
            Self::Closed(survey) => survey.graph(),
        }
    }

    pub fn graph_version(&self) -> Option<GraphVersion> {
        match self {
            Self::Draft(survey) => survey.graph_version(),
            Self::Opened(survey) => survey.graph_version(),
            Self::Closed(survey) => survey.graph_version(),
        }
    }
}

impl From<Survey<Draft>> for SurveyRecord {
    fn from(survey: Survey<Draft>) -> Self {
        Self::Draft(survey)
    }
}

impl From<Survey<Opened>> for SurveyRecord {
    fn from(survey: Survey<Opened>) -> Self {
        Self::Opened(survey)
    }
}

impl From<Survey<Closed>> for SurveyRecord {
    fn from(survey: Survey<Closed>) -> Self {
        Self::Closed(survey)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseRecord {
    InProgress(Response<InProgress>),
    Completed(Response<Completed>),
}

impl ResponseRecord {
    pub fn id(&self) -> ResponseId {
        match self {
            Self::InProgress(response) => response.id(),
            Self::Completed(response) => response.id(),
        }
    }

    pub fn survey_id(&self) -> SurveyId {
        match self {
            Self::InProgress(response) => response.survey_id(),
            Self::Completed(response) => response.survey_id(),
        }
    }

    pub fn organization_id(&self) -> OrganizationId {
        match self {
            Self::InProgress(response) => response.organization_id(),
            Self::Completed(response) => response.organization_id(),
        }
    }

    pub fn graph_version(&self) -> Option<GraphVersion> {
        match self {
            Self::InProgress(_) => None,
            Self::Completed(response) => Some(response.graph_version()),
        }
    }
}

impl From<Response<InProgress>> for ResponseRecord {
    fn from(response: Response<InProgress>) -> Self {
        Self::InProgress(response)
    }
}

impl From<Response<Completed>> for ResponseRecord {
    fn from(response: Response<Completed>) -> Self {
        Self::Completed(response)
    }
}
