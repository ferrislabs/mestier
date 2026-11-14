use std::collections::BTreeMap;

use auth::Identity;
use axum::{
    Extension, Json,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateCampaignCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    campaign::{CampaignView, CampaignWindowRequest, CampaignsPath},
    request::metadata,
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub window: CampaignWindowRequest,
    pub metadata: Option<BTreeMap<String, String>>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/survey-campaigns",
    operation_id = "createSurveyCampaign",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateCampaignRequest,
    responses(
        (status = 201, description = "Campaign created; a campaign already in its window emits survey_campaign.started", body = inline(DataEnvelope<CampaignView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Surveys are not enabled for this organization"),
        (status = 422, description = "Blank name, invalid metadata, or a window whose end precedes its start"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CampaignsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateCampaignRequest>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let window = payload.window.window()?;
    let metadata = metadata(payload.metadata)?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let campaign = state
        .usecase
        .acting_as(user_id)
        .create_campaign(CreateCampaignCommand {
            actor,
            organization_id: path.organization_id,
            name: payload.name,
            window,
            metadata,
        })
        .await?;

    Ok(Response::Created(CampaignView::from(&campaign)).into_response())
}
