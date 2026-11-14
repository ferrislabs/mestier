use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::transactional;

use crate::OrganizationId;
use crate::application::MestierUseCase;
use crate::domain::survey::ports::CampaignRepository;
use crate::domain::survey::service::CampaignService;
use crate::domain::survey::{CampaignId, CampaignStatus};

const DUE_CAMPAIGNS_PER_TICK: u64 = 100;

impl MestierUseCase {
    pub async fn advance_survey_campaigns(&self, now: DateTime<Utc>) -> Result<usize, CoreError> {
        let mut advanced = 0;
        for (organization_id, id) in self.due_survey_campaigns(now).await? {
            match self.advance_survey_campaign(organization_id, id, now).await {
                Ok(steps) if !steps.is_empty() => advanced += 1,
                Ok(_) => {}
                Err(error) => tracing::error!(
                    %error,
                    organization_id = %organization_id.0,
                    campaign_id = %id,
                    "advancing a survey campaign failed"
                ),
            }
        }
        Ok(advanced)
    }

    #[transactional(campaign)]
    async fn due_survey_campaigns(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<(OrganizationId, CampaignId)>, CoreError> {
        let mut campaign_repository = campaign_repository;
        let due = campaign_repository
            .list_due(now, DUE_CAMPAIGNS_PER_TICK)
            .await?;
        Ok(due
            .iter()
            .map(|campaign| (campaign.organization_id(), campaign.id()))
            .collect())
    }

    #[transactional(campaign, survey, emitter)]
    async fn advance_survey_campaign(
        &self,
        organization_id: OrganizationId,
        id: CampaignId,
        now: DateTime<Utc>,
    ) -> Result<Vec<CampaignStatus>, CoreError> {
        let mut service = CampaignService::new(campaign_repository, survey_repository, emitter);
        service.advance(organization_id, id, now).await
    }
}
