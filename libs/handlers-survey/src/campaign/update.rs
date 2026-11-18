use std::collections::BTreeMap;

use auth::Identity;
use axum::{
    Extension, Json,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CampaignPatch, UpdateCampaignCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    campaign::{CampaignPath, CampaignView, CampaignWindowRequest},
    request::metadata,
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub window: Option<CampaignWindowRequest>,
    pub metadata: Option<BTreeMap<String, String>>,
}

impl UpdateCampaignRequest {
    fn patch(self) -> Result<CampaignPatch, ApiError> {
        Ok(CampaignPatch {
            name: self.name,
            window: self.window.map(CampaignWindowRequest::window).transpose()?,
            metadata: self
                .metadata
                .map(|entries| metadata(Some(entries)))
                .transpose()?,
        })
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/survey-campaigns/{campaign_id}",
    operation_id = "updateSurveyCampaign",
    description = "Updates a campaign. The body shape (window, metadata) is checked before the permission, so a caller without survey.manage who sends an invalid body gets 422 rather than 403.",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("campaign_id" = mestier_core::CampaignId, Path, description = "Campaign identifier"),
    ),
    request_body = UpdateCampaignRequest,
    responses(
        (status = 200, description = "Campaign updated; a new window never moves its status back", body = inline(DataEnvelope<CampaignView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Campaign not found, or surveys not enabled"),
        (status = 422, description = "Blank name, invalid metadata, or a window whose end precedes its start"),
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
    Json(payload): Json<UpdateCampaignRequest>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let patch = payload.patch()?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let campaign = state
        .usecase
        .acting_as(user_id)
        .update_campaign(UpdateCampaignCommand {
            actor,
            organization_id,
            id: campaign_id,
            patch,
        })
        .await?;

    Ok(Response::OK(CampaignView::from(&campaign)).into_response())
}
