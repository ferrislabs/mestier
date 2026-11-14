use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_core::{
    Campaign, CampaignId, CampaignRepository, MemberId, OrganizationId, PublicSlug, ResponseId,
    ResponseRecord, ResponseRepository, ResumeToken, SurveyError, SurveyId, SurveyRecord,
    SurveyRepository,
};

#[derive(Debug, Default)]
pub struct State {
    pub surveys: HashMap<SurveyId, SurveyRecord>,
    pub campaigns: HashMap<CampaignId, Campaign>,
    pub responses: HashMap<ResponseId, ResponseRecord>,
}

pub type Store = Arc<Mutex<State>>;

pub fn state(store: &Store) -> MutexGuard<'_, State> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn page<T>(mut items: Vec<(uuid::Uuid, T)>, limit: u64, offset: u64) -> (Vec<T>, u64) {
    items.sort_by_key(|(id, _)| std::cmp::Reverse(*id));
    let total = items.len() as u64;
    let kept = items
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|(_, item)| item)
        .collect();
    (kept, total)
}

fn campaign_of(record: &SurveyRecord) -> Option<CampaignId> {
    match record {
        SurveyRecord::Draft(survey) => survey.campaign_id(),
        SurveyRecord::Opened(survey) => survey.campaign_id(),
        SurveyRecord::Closed(survey) => survey.campaign_id(),
    }
}

fn slug_of(record: &SurveyRecord) -> Option<&PublicSlug> {
    match record {
        SurveyRecord::Draft(_) => None,
        SurveyRecord::Opened(survey) => survey.public_slug(),
        SurveyRecord::Closed(survey) => survey.public_slug(),
    }
}

fn stage_of(record: &SurveyRecord) -> u8 {
    match record {
        SurveyRecord::Draft(_) => 0,
        SurveyRecord::Opened(_) => 1,
        SurveyRecord::Closed(_) => 2,
    }
}

fn member_of(record: &ResponseRecord) -> Option<MemberId> {
    match record {
        ResponseRecord::InProgress(response) => response.member_id(),
        ResponseRecord::Completed(response) => response.member_id(),
    }
}

fn token_of(record: &ResponseRecord) -> Option<&ResumeToken> {
    match record {
        ResponseRecord::InProgress(response) => response.resume_token(),
        ResponseRecord::Completed(response) => response.resume_token(),
    }
}

fn identifier_of(record: &ResponseRecord) -> Option<&str> {
    match record {
        ResponseRecord::InProgress(_) => None,
        ResponseRecord::Completed(response) => response.identifier_value(),
    }
}

fn is_unique_completed(record: &ResponseRecord) -> bool {
    match record {
        ResponseRecord::InProgress(_) => false,
        ResponseRecord::Completed(response) => response.unique_respondent(),
    }
}

fn is_completed(record: &ResponseRecord) -> bool {
    matches!(record, ResponseRecord::Completed(_))
}

#[derive(Debug, Clone)]
pub struct MemorySurveys {
    store: Store,
}

#[derive(Debug, Clone)]
pub struct MemoryCampaigns {
    store: Store,
}

#[derive(Debug, Clone)]
pub struct MemoryResponses {
    store: Store,
}

pub fn surveys(store: &Store) -> MemorySurveys {
    MemorySurveys {
        store: store.clone(),
    }
}

pub fn campaigns(store: &Store) -> MemoryCampaigns {
    MemoryCampaigns {
        store: store.clone(),
    }
}

pub fn responses(store: &Store) -> MemoryResponses {
    MemoryResponses {
        store: store.clone(),
    }
}

impl SurveyRepository for MemorySurveys {
    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<Option<SurveyRecord>, CoreError> {
        Ok(state(&self.store)
            .surveys
            .get(&id)
            .filter(|record| record.organization_id() == organization_id)
            .cloned())
    }

    async fn insert(&mut self, survey: &SurveyRecord) -> Result<SurveyRecord, CoreError> {
        let mut guard = state(&self.store);
        let slug_taken = slug_of(survey).is_some_and(|slug| {
            guard
                .surveys
                .values()
                .any(|other| slug_of(other) == Some(slug))
        });
        if slug_taken || guard.surveys.contains_key(&survey.id()) {
            return Err(CoreError::Conflict("survey: already stored".to_owned()));
        }
        guard.surveys.insert(survey.id(), survey.clone());
        Ok(survey.clone())
    }

    async fn update(&mut self, survey: &SurveyRecord) -> Result<SurveyRecord, CoreError> {
        let mut guard = state(&self.store);
        let stored = guard
            .surveys
            .get(&survey.id())
            .filter(|stored| stored.organization_id() == survey.organization_id())
            .ok_or(CoreError::NotFound)?;
        if stage_of(stored) > stage_of(survey) {
            return Err(SurveyError::SurveyStateChanged.into());
        }
        guard.surveys.insert(survey.id(), survey.clone());
        Ok(survey.clone())
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SurveyRecord>, u64), CoreError> {
        let items = state(&self.store)
            .surveys
            .values()
            .filter(|record| record.organization_id() == organization_id)
            .map(|record| (record.id().0, record.clone()))
            .collect();
        Ok(page(items, limit, offset))
    }

    async fn list_by_campaign(
        &mut self,
        organization_id: OrganizationId,
        campaign_id: CampaignId,
    ) -> Result<Vec<SurveyRecord>, CoreError> {
        Ok(state(&self.store)
            .surveys
            .values()
            .filter(|record| record.organization_id() == organization_id)
            .filter(|record| campaign_of(record) == Some(campaign_id))
            .cloned()
            .collect())
    }

    async fn find_by_public_slug(
        &mut self,
        slug: &PublicSlug,
    ) -> Result<Option<SurveyRecord>, CoreError> {
        Ok(state(&self.store)
            .surveys
            .values()
            .find(|record| slug_of(record) == Some(slug))
            .cloned())
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<(), CoreError> {
        let mut guard = state(&self.store);
        if guard
            .surveys
            .get(&id)
            .is_some_and(|record| record.organization_id() == organization_id)
        {
            guard.surveys.remove(&id);
        }
        Ok(())
    }

    async fn has_responses(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<bool, CoreError> {
        Ok(state(&self.store)
            .responses
            .values()
            .any(|record| record.organization_id() == organization_id && record.survey_id() == id))
    }

    async fn find_for_update(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<Option<SurveyRecord>, CoreError> {
        self.find(organization_id, id).await
    }
}

impl CampaignRepository for MemoryCampaigns {
    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
    ) -> Result<Option<Campaign>, CoreError> {
        Ok(state(&self.store)
            .campaigns
            .get(&id)
            .filter(|campaign| campaign.organization_id() == organization_id)
            .cloned())
    }

    async fn insert(&mut self, campaign: &Campaign) -> Result<Campaign, CoreError> {
        let mut guard = state(&self.store);
        if guard.campaigns.contains_key(&campaign.id()) {
            return Err(CoreError::Conflict("campaign: already stored".to_owned()));
        }
        guard.campaigns.insert(campaign.id(), campaign.clone());
        Ok(campaign.clone())
    }

    async fn update(&mut self, campaign: &Campaign) -> Result<Campaign, CoreError> {
        let mut guard = state(&self.store);
        let owned = guard
            .campaigns
            .get(&campaign.id())
            .is_some_and(|stored| stored.organization_id() == campaign.organization_id());
        if !owned {
            return Err(CoreError::NotFound);
        }
        guard.campaigns.insert(campaign.id(), campaign.clone());
        Ok(campaign.clone())
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Campaign>, u64), CoreError> {
        let items = state(&self.store)
            .campaigns
            .values()
            .filter(|campaign| campaign.organization_id() == organization_id)
            .map(|campaign| (campaign.id().0, campaign.clone()))
            .collect();
        Ok(page(items, limit, offset))
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
    ) -> Result<(), CoreError> {
        let mut guard = state(&self.store);
        if guard
            .campaigns
            .get(&id)
            .is_some_and(|campaign| campaign.organization_id() == organization_id)
        {
            guard.campaigns.remove(&id);
        }
        Ok(())
    }

    async fn list_due(
        &mut self,
        now: DateTime<Utc>,
        limit: u64,
    ) -> Result<Vec<Campaign>, CoreError> {
        Ok(state(&self.store)
            .campaigns
            .values()
            .filter(|campaign| campaign.status_at(now) > campaign.status())
            .take(limit as usize)
            .cloned()
            .collect())
    }

    async fn find_for_update(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
    ) -> Result<Option<Campaign>, CoreError> {
        self.find(organization_id, id).await
    }
}

fn already_complete() -> CoreError {
    SurveyError::ResponseAlreadyCompleted.into()
}

impl ResponseRepository for MemoryResponses {
    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: ResponseId,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        Ok(state(&self.store)
            .responses
            .get(&id)
            .filter(|record| record.organization_id() == organization_id)
            .cloned())
    }

    async fn insert(&mut self, response: &ResponseRecord) -> Result<ResponseRecord, CoreError> {
        let mut guard = state(&self.store);
        let token_taken = token_of(response).is_some_and(|token| {
            guard
                .responses
                .values()
                .any(|other| token_of(other) == Some(token))
        });
        if token_taken || guard.responses.contains_key(&response.id()) {
            return Err(CoreError::Conflict("response: already stored".to_owned()));
        }
        guard.responses.insert(response.id(), response.clone());
        Ok(response.clone())
    }

    async fn update(&mut self, response: &ResponseRecord) -> Result<ResponseRecord, CoreError> {
        let mut guard = state(&self.store);
        let stored = guard
            .responses
            .get(&response.id())
            .filter(|stored| stored.organization_id() == response.organization_id())
            .ok_or(CoreError::NotFound)?;
        if is_completed(stored) {
            return Err(already_complete());
        }
        if is_unique_completed(response) {
            let clash = guard.responses.values().any(|other| {
                other.id() != response.id()
                    && other.survey_id() == response.survey_id()
                    && is_unique_completed(other)
                    && (member_of(response).is_some() && member_of(other) == member_of(response)
                        || identifier_of(response).is_some()
                            && identifier_of(other) == identifier_of(response))
            });
            if clash {
                return Err(SurveyError::RespondentAlreadyAnswered.into());
            }
        }
        guard.responses.insert(response.id(), response.clone());
        Ok(response.clone())
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ResponseRecord>, u64), CoreError> {
        let items = state(&self.store)
            .responses
            .values()
            .filter(|record| record.organization_id() == organization_id)
            .map(|record| (record.id().0, record.clone()))
            .collect();
        Ok(page(items, limit, offset))
    }

    async fn list_for_survey(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ResponseRecord>, u64), CoreError> {
        let items = state(&self.store)
            .responses
            .values()
            .filter(|record| {
                record.organization_id() == organization_id && record.survey_id() == survey_id
            })
            .map(|record| (record.id().0, record.clone()))
            .collect();
        Ok(page(items, limit, offset))
    }

    async fn find_by_resume_token(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        token: &ResumeToken,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        Ok(state(&self.store)
            .responses
            .values()
            .find(|record| {
                record.organization_id() == organization_id
                    && record.survey_id() == survey_id
                    && token_of(record) == Some(token)
            })
            .cloned())
    }

    async fn count_completed(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
    ) -> Result<u64, CoreError> {
        Ok(state(&self.store)
            .responses
            .values()
            .filter(|record| {
                record.organization_id() == organization_id
                    && record.survey_id() == survey_id
                    && is_completed(record)
            })
            .count() as u64)
    }

    async fn find_completed_by_member(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        member_id: MemberId,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        Ok(state(&self.store)
            .responses
            .values()
            .find(|record| {
                record.organization_id() == organization_id
                    && record.survey_id() == survey_id
                    && is_completed(record)
                    && member_of(record) == Some(member_id)
            })
            .cloned())
    }

    async fn find_completed_by_identifier(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        value: &str,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        Ok(state(&self.store)
            .responses
            .values()
            .find(|record| {
                record.organization_id() == organization_id
                    && record.survey_id() == survey_id
                    && identifier_of(record) == Some(value)
            })
            .cloned())
    }

    async fn find_in_progress_by_member(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        member_id: MemberId,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        Ok(state(&self.store)
            .responses
            .values()
            .find(|record| {
                record.organization_id() == organization_id
                    && record.survey_id() == survey_id
                    && !is_completed(record)
                    && member_of(record) == Some(member_id)
            })
            .cloned())
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: ResponseId,
    ) -> Result<(), CoreError> {
        let mut guard = state(&self.store);
        if guard
            .responses
            .get(&id)
            .is_some_and(|record| record.organization_id() == organization_id)
        {
            guard.responses.remove(&id);
        }
        Ok(())
    }
}
