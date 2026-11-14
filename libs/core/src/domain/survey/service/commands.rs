use std::num::NonZeroU32;

use crate::MemberId;
use crate::domain::survey::{
    Graph, InProgress, Metadata, Response, ResponseId, ResumeToken, SurveyId, Visibility, Window,
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

impl ResponseLocator {
    pub fn visibility(&self) -> Visibility {
        match self {
            Self::Member { .. } => Visibility::Members,
            Self::Public { .. } => Visibility::Public,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StartedResponse {
    Created(Response<InProgress>),
    Resumed(Response<InProgress>),
}

impl StartedResponse {
    pub fn is_resumed(&self) -> bool {
        matches!(self, Self::Resumed(_))
    }

    pub fn response(&self) -> &Response<InProgress> {
        match self {
            Self::Created(response) | Self::Resumed(response) => response,
        }
    }

    pub fn into_response(self) -> Response<InProgress> {
        match self {
            Self::Created(response) | Self::Resumed(response) => response,
        }
    }
}
