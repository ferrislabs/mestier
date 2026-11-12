use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{SearchPattern, TriadFilter};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{paths::TriadCollectionPath, require_view_cursus, response::TriadSummaryResponse};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListTriadsQuery {
    pub promotion_id: Option<mestier_core::PromotionId>,
    /// Absent keeps every triad, `true` only closed ones, `false` only open
    /// ones.
    #[param(example = false)]
    pub closed: Option<bool>,
    /// A case-insensitive **substring** of the apprentice name (`first last` or `last first`), the current host company name or the promotion label. Not full-text
    /// search: no stemming, no ranking, no word boundaries. The value is
    /// trimmed, and `%`, `_` and `\` match themselves. Empty after trimming
    /// means no filter, which is what a cleared search box means.
    pub q: Option<String>,
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
        (status = 200, description = "A page of triads narrowed by promotion, `closed` and `q` (`total` counts the filtered rows), with apprentice, promotion, pedagogic tutor and current host company names", body = inline(DataEnvelope<Vec<TriadSummaryResponse>>)),
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
            TriadFilter {
                promotion_id: query.promotion_id,
                closed: query.closed,
                search: SearchPattern::parse(query.q.as_deref()),
            },
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
