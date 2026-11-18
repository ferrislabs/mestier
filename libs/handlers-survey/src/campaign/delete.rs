use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};
use mestier_core::CampaignCommand;

use crate::{campaign::CampaignPath, require_org_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/survey-campaigns/{campaign_id}",
    operation_id = "deleteSurveyCampaign",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("campaign_id" = mestier_core::CampaignId, Path, description = "Campaign identifier"),
    ),
    responses(
        (status = 204, description = "Campaign deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Campaign not found, or surveys not enabled"),
        (status = 409, description = "Surveys are still attached to the campaign"),
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
) -> Result<Response<()>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_campaign(CampaignCommand {
            actor,
            organization_id,
            id: campaign_id,
        })
        .await?;

    Ok(Response::NoContent)
}
