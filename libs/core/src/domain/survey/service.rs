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
pub use response_service::ResponseService;
pub use survey_service::SurveyService;

async fn campaign_of<C: CampaignRepository>(
    campaigns: &mut C,
    organization_id: OrganizationId,
    campaign_id: Option<CampaignId>,
) -> Result<Option<Campaign>, CoreError> {
    let Some(campaign_id) = campaign_id else {
        return Ok(None);
    };
    campaigns
        .find(organization_id, campaign_id)
        .await?
        .map(Some)
        .ok_or_else(|| SurveyError::CampaignNotFound.into())
}
