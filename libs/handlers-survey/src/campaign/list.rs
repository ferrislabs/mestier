use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    resolve_actor,
};
use mestier_core::ListCampaignsCommand;

use crate::{
    campaign::{CampaignView, CampaignsPath},
    require_org_membership,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/survey-campaigns",
    operation_id = "listSurveyCampaigns",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated campaigns of the organization", body = inline(DataEnvelope<Vec<CampaignView>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.view or survey.manage"),
        (status = 404, description = "Surveys are not enabled for this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CampaignsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let per_page = pagination.per_page();
    let (campaigns, total) = state
        .usecase
        .acting_as(user_id)
        .list_campaigns(ListCampaignsCommand {
            actor,
            organization_id: path.organization_id,
            limit: per_page,
            offset: pagination.offset(),
        })
        .await?;
    let items: Vec<CampaignView<'_>> = campaigns.iter().map(CampaignView::from).collect();
    let meta = PaginationMetadata::new(per_page, pagination.page(), Some(total), items.is_empty());

    Ok(Response::Paginated(Page::new(items, meta)).into_response())
}
