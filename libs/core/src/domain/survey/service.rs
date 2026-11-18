use common::CoreError;

use super::ports::CampaignRepository;
use super::{Campaign, CampaignId, SurveyError};
use crate::OrganizationId;

mod campaign_service;
mod commands;
mod response_service;
mod survey_service;

pub use campaign_service::CampaignService;
pub use commands::{CampaignPatch, ResponseLocator, StartedResponse, SurveyPatch};
pub use response_service::{MAX_UPLOADED_PHOTOS, ResponseService};
pub use survey_service::SurveyService;

async fn campaigns_of<C: CampaignRepository>(
    campaigns: &mut C,
    organization_id: OrganizationId,
    campaign_ids: &[CampaignId],
) -> Result<Vec<Campaign>, CoreError> {
    let mut found = Vec::with_capacity(campaign_ids.len());
    for campaign_id in campaign_ids {
        let campaign = campaigns
            .find(organization_id, *campaign_id)
            .await?
            .ok_or(SurveyError::CampaignNotFound)?;
        found.push(campaign);
    }
    Ok(found)
}
