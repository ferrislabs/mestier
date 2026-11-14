use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub mod aggregate;
pub mod campaign;
pub mod error;
pub mod graph;
pub mod metadata;
pub mod response;
pub mod value;

#[cfg(test)]
mod tests;

pub use aggregate::{Closed, Draft, Opened, Survey};
pub use campaign::{Campaign, CampaignStatus, Window};
pub use error::SurveyError;
pub use graph::{
    Answers, Branch, ConditionNode, Edge, EndNode, Graph, GraphPath, Node, NodeRef, QuestionNode,
};
pub use metadata::Metadata;
pub use response::{Completed, InProgress, Response, ResumeToken};
pub use value::{Answer, EmailAddress, PhoneNumber, QuestionId, QuestionKind, Visibility};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct SurveyId(pub Uuid);

impl FromStr for SurveyId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(SurveyId)
    }
}

impl Display for SurveyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct CampaignId(pub Uuid);

impl FromStr for CampaignId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(CampaignId)
    }
}

impl Display for CampaignId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ResponseId(pub Uuid);

impl FromStr for ResponseId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ResponseId)
    }
}

impl Display for ResponseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
