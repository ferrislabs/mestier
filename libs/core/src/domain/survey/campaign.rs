use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CampaignStatus {
    Scheduled,
    Running,
    Ended,
}

impl CampaignStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "SCHEDULED",
            Self::Running => "RUNNING",
            Self::Ended => "ENDED",
        }
    }

    fn of(window: &Window, now: DateTime<Utc>) -> Self {
        if window.starts_at.is_some_and(|start| now < start) {
            Self::Scheduled
        } else if window.ends_at.is_some_and(|end| now >= end) {
            Self::Ended
        } else {
            Self::Running
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CampaignDraft")]
pub struct Campaign {
    id: CampaignId,
    organization_id: OrganizationId,
    name: String,
    window: Window,
    metadata: Metadata,
    status: CampaignStatus,
}

#[derive(Deserialize)]
struct CampaignDraft {
    id: CampaignId,
    organization_id: OrganizationId,
    name: String,
    window: Window,
    metadata: Metadata,
    status: CampaignStatus,
}

impl TryFrom<CampaignDraft> for Campaign {
    type Error = SurveyError;

    fn try_from(draft: CampaignDraft) -> Result<Self, Self::Error> {
        check_name(&draft.name)?;
        Ok(Self {
            id: draft.id,
            organization_id: draft.organization_id,
            name: draft.name,
            window: draft.window,
            metadata: draft.metadata,
            status: draft.status,
        })
    }
}

fn check_name(name: &str) -> Result<(), SurveyError> {
    if name.trim().is_empty() {
        Err(SurveyError::CampaignNameEmpty)
    } else {
        Ok(())
    }
}

impl Campaign {
    pub fn new(
        id: CampaignId,
        organization_id: OrganizationId,
        name: String,
        window: Window,
        metadata: Metadata,
        now: DateTime<Utc>,
    ) -> Result<Self, SurveyError> {
        check_name(&name)?;
        Ok(Self {
            id,
            organization_id,
            name,
            status: CampaignStatus::of(&window, now),
            window,
            metadata,
        })
    }

    pub fn id(&self) -> CampaignId {
        self.id
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn status(&self) -> CampaignStatus {
        self.status
    }

    pub fn status_at(&self, now: DateTime<Utc>) -> CampaignStatus {
        CampaignStatus::of(&self.window, now)
    }

    pub fn advance(&mut self, now: DateTime<Utc>) -> Option<CampaignStatus> {
        let target = self.status_at(now);
        if target > self.status {
            self.status = target;
            Some(target)
        } else {
            None
        }
    }

    pub fn rename(&mut self, name: String) -> Result<(), SurveyError> {
        check_name(&name)?;
        self.name = name;
        Ok(())
    }

    pub fn reschedule(&mut self, window: Window, now: DateTime<Utc>) -> Option<CampaignStatus> {
        self.window = window;
        self.advance(now)
    }

    pub fn edit_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }
}
