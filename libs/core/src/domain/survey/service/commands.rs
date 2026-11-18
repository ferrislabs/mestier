use std::num::NonZeroU32;

use crate::MemberId;
use crate::domain::survey::{
    Graph, Metadata, ResponseId, ResumeToken, SurveyId, Visibility, Window,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SurveyPatch {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub visibility: Option<Visibility>,
    pub allow_multiple_responses: Option<bool>,
    pub response_cap: Option<Option<NonZeroU32>>,
    pub metadata: Option<Metadata>,
    pub graph: Option<Option<Graph>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CampaignPatch {
    pub name: Option<String>,
    pub window: Option<Window>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseLocator {
    Member {
        id: ResponseId,
        member_id: MemberId,
    },
    Public {
        survey_id: SurveyId,
        token: ResumeToken,
    },
}
