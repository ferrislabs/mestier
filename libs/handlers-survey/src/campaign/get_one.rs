use auth::Identity;
use axum::{
    Extension,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CampaignCommand;

use crate::{
    campaign::{CampaignPath, CampaignView},
    require_org_membership,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/survey-campaigns/{campaign_id}",
    operation_id = "getSurveyCampaign",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("campaign_id" = mestier_core::CampaignId, Path, description = "Campaign identifier"),
    ),
    responses(
        (status = 200, description = "The campaign", body = inline(DataEnvelope<CampaignView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.view or survey.manage"),
        (status = 404, description = "Campaign not found, or surveys not enabled"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CampaignPath {
        organization_id,
        campaign_id,
    }: CampaignPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let campaign = state
        .usecase
        .acting_as(user_id)
        .get_campaign(CampaignCommand {
            actor,
            organization_id,
            id: campaign_id,
        })
        .await?;

    Ok(Response::OK(CampaignView::from(&campaign)).into_response())
}
