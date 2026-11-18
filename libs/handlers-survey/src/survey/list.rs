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
use mestier_core::{CampaignId, ListSurveysCommand};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    require_org_membership,
    survey::{SurveyView, SurveysPath},
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SurveyListFilter {
    campaign_id: Option<CampaignId>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/surveys",
    operation_id = "listSurveys",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
        SurveyListFilter,
    ),
    responses(
        (status = 200, description = "Paginated surveys of the organization, only those of a campaign when campaign_id is given", body = inline(DataEnvelope<Vec<SurveyView>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.view or survey.manage"),
        (status = 404, description = "Surveys are not enabled for this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: SurveysPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
    Query(filter): Query<SurveyListFilter>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let per_page = pagination.per_page();
    let (surveys, total) = state
        .usecase
        .acting_as(user_id)
        .list_surveys(ListSurveysCommand {
            actor,
            organization_id: path.organization_id,
            campaign_id: filter.campaign_id,
            limit: per_page,
            offset: pagination.offset(),
        })
        .await?;
    let items: Vec<SurveyView<'_>> = surveys.iter().map(SurveyView::from).collect();
    let meta = PaginationMetadata::new(per_page, pagination.page(), Some(total), items.is_empty());

    Ok(Response::Paginated(Page::new(items, meta)).into_response())
}
