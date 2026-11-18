use chrono::{DateTime, Utc};
use common::CoreError;
use events::EventEmitter;

use super::CampaignPatch;
use crate::OrganizationId;
use crate::domain::survey::events::{SurveyCampaignEnded, SurveyCampaignStarted};
use crate::domain::survey::ports::{CampaignRepository, SurveyRepository};
use crate::domain::survey::{Campaign, CampaignId, CampaignStatus, Metadata, SurveyError, Window};

pub struct CampaignService<C, S, E>
where
    C: CampaignRepository,
    S: SurveyRepository,
    E: EventEmitter,
{
    campaigns: C,
    surveys: S,
    emitter: E,
}

fn steps(before: CampaignStatus, after: Option<CampaignStatus>) -> Vec<CampaignStatus> {
    match (before, after) {
        (_, None) => Vec::new(),
        (CampaignStatus::Scheduled, Some(CampaignStatus::Ended)) => {
            vec![CampaignStatus::Running, CampaignStatus::Ended]
        }
        (_, Some(status)) => vec![status],
    }
}

impl<C, S, E> CampaignService<C, S, E>
where
    C: CampaignRepository,
    S: SurveyRepository,
    E: EventEmitter,
{
    pub fn new(campaigns: C, surveys: S, emitter: E) -> Self {
        Self {
            campaigns,
            surveys,
            emitter,
        }
    }

    fn emit_steps(&self, campaign: &Campaign, steps: &[CampaignStatus]) -> Result<(), CoreError> {
        for step in steps {
            match step {
                CampaignStatus::Running => self.emitter.emit(
                    campaign.organization_id(),
                    &SurveyCampaignStarted {
                        campaign: campaign.clone(),
                    },
                )?,
                CampaignStatus::Ended => self.emitter.emit(
                    campaign.organization_id(),
                    &SurveyCampaignEnded {
                        campaign: campaign.clone(),
                    },
                )?,
                CampaignStatus::Scheduled => {}
            }
        }
        Ok(())
    }

    async fn find_locked(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
    ) -> Result<Campaign, CoreError> {
        self.campaigns
            .find_for_update(organization_id, id)
            .await?
            .ok_or_else(|| SurveyError::CampaignNotFound.into())
    }

    pub async fn create(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
        name: String,
        window: Window,
        metadata: Metadata,
        now: DateTime<Utc>,
    ) -> Result<Campaign, CoreError> {
        let campaign = Campaign::new(id, organization_id, name, window, metadata, now)?;
        let saved = self.campaigns.insert(&campaign).await?;
        let status = saved.status();
        let steps = steps(
            CampaignStatus::Scheduled,
            (status != CampaignStatus::Scheduled).then_some(status),
        );
        self.emit_steps(&saved, &steps)?;
        Ok(saved)
    }

    pub async fn update(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
        patch: CampaignPatch,
        now: DateTime<Utc>,
    ) -> Result<Campaign, CoreError> {
        let mut campaign = self.find_locked(organization_id, id).await?;
        let before = campaign.status();
        if let Some(name) = patch.name {
            campaign.rename(name)?;
        }
        if let Some(metadata) = patch.metadata {
            campaign.edit_metadata(metadata);
        }
        let after = match patch.window {
            Some(window) => campaign.reschedule(window, now),
            None => campaign.advance(now),
        };
        let saved = self.campaigns.update(&campaign).await?;
        self.emit_steps(&saved, &steps(before, after))?;
        Ok(saved)
    }

    pub async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
    ) -> Result<(), CoreError> {
        self.find_locked(organization_id, id).await?;
        let (_, attached) = self
            .surveys
            .list_summaries_by_campaign(organization_id, id, 1, 0)
            .await?;
        if attached > 0 {
            return Err(SurveyError::CampaignHasSurveys.into());
        }
        self.campaigns.delete(organization_id, id).await
    }

    pub async fn advance(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
        now: DateTime<Utc>,
    ) -> Result<Vec<CampaignStatus>, CoreError> {
        let Some(mut campaign) = self.campaigns.find_for_update(organization_id, id).await? else {
            return Ok(Vec::new());
        };
        let steps = steps(campaign.status(), campaign.advance(now));
        if steps.is_empty() {
            return Ok(steps);
        }
        let saved = self.campaigns.update(&campaign).await?;
        self.emit_steps(&saved, &steps)?;
        Ok(steps)
    }
}
