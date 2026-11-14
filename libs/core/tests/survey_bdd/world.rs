use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};
use common::CoreError;
use events::testing::RecordingEmitter;
use mestier_core::{
    CampaignId, CampaignService, MemberId, OrganizationId, QuestionId, ResponseId, ResponseService,
    ResumeToken, SurveyId, SurveyService,
};

use crate::repository::{self, MemoryCampaigns, MemoryResponses, MemorySurveys, Store};

pub type Surveys<'a> = SurveyService<MemorySurveys, MemoryCampaigns, &'a RecordingEmitter>;

pub type Campaigns<'a> = CampaignService<MemoryCampaigns, MemorySurveys, &'a RecordingEmitter>;

pub type Responses<'a> =
    ResponseService<MemoryResponses, MemorySurveys, MemoryCampaigns, &'a RecordingEmitter>;

#[derive(Debug, Clone)]
pub enum Respondent {
    Member(MemberId),
    Visitor(ResumeToken),
}

#[derive(Debug, Default, cucumber::World)]
pub struct SurveyWorld {
    pub store: Store,
    pub emitter: RecordingEmitter,
    pub organization_id: Option<OrganizationId>,
    pub other_organization_id: Option<OrganizationId>,
    pub clock: Option<DateTime<Utc>>,
    pub respondents: HashMap<String, Respondent>,
    pub surveys: HashMap<String, SurveyId>,
    pub campaigns: HashMap<String, CampaignId>,
    pub questions: HashMap<(String, String), QuestionId>,
    pub responses: HashMap<(String, String), ResponseId>,
    pub resumed: Option<ResponseId>,
    pub listed: Option<usize>,
    pub refusal: Option<CoreError>,
}

impl SurveyWorld {
    pub fn surveys(&self) -> Surveys<'_> {
        SurveyService::new(
            repository::surveys(&self.store),
            repository::campaigns(&self.store),
            &self.emitter,
        )
    }

    pub fn campaign_service(&self) -> Campaigns<'_> {
        CampaignService::new(
            repository::campaigns(&self.store),
            repository::surveys(&self.store),
            &self.emitter,
        )
    }

    pub fn responses(&self) -> Responses<'_> {
        ResponseService::new(
            repository::responses(&self.store),
            repository::surveys(&self.store),
            repository::campaigns(&self.store),
            &self.emitter,
        )
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
            .expect("a scenario states the company in its `Given`")
    }

    pub fn other_organization_id(&self) -> OrganizationId {
        self.other_organization_id
            .expect("a scenario states the other company in its `Given`")
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.clock
            .unwrap_or_else(|| Utc.with_ymd_and_hms(2026, 3, 10, 12, 0, 0).unwrap())
    }
}
