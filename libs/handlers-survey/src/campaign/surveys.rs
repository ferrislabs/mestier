use auth::Identity;
use axum::{
    Extension,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CampaignCommand;

use crate::{campaign::CampaignSurveysPath, require_org_membership, survey::SurveySummaryView};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/survey-campaigns/{campaign_id}/surveys",
    operation_id = "listSurveyCampaignSurveys",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("campaign_id" = mestier_core::CampaignId, Path, description = "Campaign identifier"),
    ),
    responses(
        (status = 200, description = "The surveys of the campaign, the most recent first", body = inline(DataEnvelope<Vec<SurveySummaryView>>)),
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
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let surveys = state
        .usecase
        .acting_as(user_id)
        .list_campaign_surveys(CampaignCommand {
            actor,
            organization_id,
            id: campaign_id,
        })
        .await?;
    let items: Vec<SurveySummaryView<'_>> = surveys.iter().map(SurveySummaryView::from).collect();

    Ok(Response::OK(items).into_response())
}
