use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState};
use mestier_core::{Campaign, CampaignId, CampaignStatus, Metadata, OrganizationId, SurveyWindow};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::request::refused;

pub mod create;
pub mod delete;
pub mod get_one;
pub mod list;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/survey-campaigns")]
pub struct CampaignsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/survey-campaigns/{campaign_id}")]
pub struct CampaignPath {
    pub organization_id: OrganizationId,
    pub campaign_id: CampaignId,
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
pub struct CampaignWindowRequest {
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
}

impl CampaignWindowRequest {
    fn window(self) -> Result<SurveyWindow, ApiError> {
        SurveyWindow::new(self.starts_at, self.ends_at).map_err(refused)
    }
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct CampaignView<'a> {
    pub id: CampaignId,
    pub organization_id: OrganizationId,
    pub name: &'a str,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub status: CampaignStatus,
    #[schema(value_type = BTreeMap<String, String>)]
    pub metadata: &'a Metadata,
}

impl<'a> From<&'a Campaign> for CampaignView<'a> {
    fn from(campaign: &'a Campaign) -> Self {
        Self {
            id: campaign.id(),
            organization_id: campaign.organization_id(),
            name: campaign.name(),
            starts_at: campaign.window().starts_at(),
            ends_at: campaign.window().ends_at(),
            status: campaign.status(),
            metadata: campaign.metadata(),
        }
    }
}
