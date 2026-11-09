use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{paths::TriadCollectionPath, require_view_cursus, response::TriadSummaryResponse};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListTriadsQuery {
    pub promotion_id: Option<mestier_core::PromotionId>,
    #[param(example = false)]
    pub closed: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/triads",
    operation_id = "listCursusTriads",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListTriadsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "A page of triads with apprentice, promotion, pedagogic tutor and current host company names", body = inline(DataEnvelope<Vec<TriadSummaryResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TriadCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListTriadsQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<TriadSummaryResponse>, ApiError> {
    require_view_cursus(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let current_page = pagination.page();
    let (items, total) = state
        .usecase
        .list_triads(
            path.organization_id,
            query.promotion_id,
            query.closed,
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<TriadSummaryResponse> =
        items.into_iter().map(TriadSummaryResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, current_page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
