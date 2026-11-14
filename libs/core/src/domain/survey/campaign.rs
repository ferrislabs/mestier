use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::CampaignId;
use super::error::SurveyError;
use super::metadata::Metadata;
use crate::OrganizationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "WindowDraft")]
pub struct Window {
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct WindowDraft {
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
}

impl TryFrom<WindowDraft> for Window {
    type Error = SurveyError;

    fn try_from(draft: WindowDraft) -> Result<Self, Self::Error> {
        Self::new(draft.starts_at, draft.ends_at)
    }
}

impl Window {
    pub fn new(
        starts_at: Option<DateTime<Utc>>,
        ends_at: Option<DateTime<Utc>>,
    ) -> Result<Self, SurveyError> {
        match (starts_at, ends_at) {
            (Some(start), Some(end)) if start >= end => Err(SurveyError::WindowInverted),
            _ => Ok(Self { starts_at, ends_at }),
        }
    }

    pub fn starts_at(&self) -> Option<DateTime<Utc>> {
        self.starts_at
    }

    pub fn ends_at(&self) -> Option<DateTime<Utc>> {
        self.ends_at
    }

    pub fn contains(&self, now: DateTime<Utc>) -> bool {
        self.starts_at.is_none_or(|start| now >= start) && self.ends_at.is_none_or(|end| now < end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CampaignStatus {
    Scheduled,
    Running,
    Ended,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Campaign {
    pub id: CampaignId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub window: Window,
    pub metadata: Metadata,
}

impl Campaign {
    pub fn status_at(&self, now: DateTime<Utc>) -> CampaignStatus {
        if self.window.starts_at.is_some_and(|start| now < start) {
            CampaignStatus::Scheduled
        } else if self.window.ends_at.is_some_and(|end| now >= end) {
            CampaignStatus::Ended
        } else {
            CampaignStatus::Running
        }
    }
}
