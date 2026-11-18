use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, Response, resolve_actor,
};
use mestier_core::ListCampaignSurveysCommand;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{campaign::CampaignSurveysPath, require_org_membership, survey::SurveySummaryView};

pub const DEFAULT_PER_PAGE: u64 = 50;
pub const MAX_PER_PAGE: u64 = 200;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CampaignSurveysPage {
    #[param(minimum = 1)]
    page: Option<u64>,
    #[param(minimum = 1, maximum = 200)]
    per_page: Option<u64>,
}

impl CampaignSurveysPage {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> u64 {
        self.per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE)
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1).saturating_mul(self.per_page())
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/survey-campaigns/{campaign_id}/surveys",
    operation_id = "listSurveyCampaignSurveys",
    tag = super::super::TAG,
    description = "Lists a summary (id, title, status) of the surveys attached to a campaign, the most recent first, 50 per page by default and 200 at most.",
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("campaign_id" = mestier_core::CampaignId, Path, description = "Campaign identifier"),
        CampaignSurveysPage,
    ),
    responses(
        (status = 200, description = "Paginated surveys of the campaign, the most recent first", body = inline(DataEnvelope<Vec<SurveySummaryView>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.view or survey.manage"),
        (status = 404, description = "Campaign not found, or surveys not enabled"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CampaignSurveysPath {
        organization_id,
        campaign_id,
    }: CampaignSurveysPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<CampaignSurveysPage>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let per_page = pagination.per_page();
    let (surveys, total) = state
        .usecase
        .acting_as(user_id)
        .list_campaign_surveys(ListCampaignSurveysCommand {
            actor,
            organization_id,
            id: campaign_id,
            limit: per_page,
            offset: pagination.offset(),
        })
        .await?;
    let items: Vec<SurveySummaryView<'_>> = surveys.iter().map(SurveySummaryView::from).collect();
    let meta = PaginationMetadata::new(per_page, pagination.page(), Some(total), items.is_empty());

    Ok(Response::Paginated(Page::new(items, meta)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_campaign_lists_fifty_surveys_by_default_and_two_hundred_at_most() {
        let default = CampaignSurveysPage::default();
        assert_eq!(default.per_page(), 50);
        assert_eq!(default.offset(), 0);
        let wide = CampaignSurveysPage {
            page: Some(3),
            per_page: Some(1000),
        };
        assert_eq!(wide.per_page(), 200);
        assert_eq!(wide.offset(), 400);
    }
}
