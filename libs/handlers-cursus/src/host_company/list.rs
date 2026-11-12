use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{HostCompanyFilter, SearchPattern};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{paths::HostCompanyCollectionPath, require_view_cursus, response::HostCompanyResponse};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListHostCompaniesQuery {
    /// Which rows to return by archive state: `active` (the default) keeps
    /// rows with no `archived_at`, `archived` keeps only archived rows, `all`
    /// keeps both.
    #[param(inline, example = "active")]
    pub archived: Option<mestier_core::ArchivedFilter>,
    /// A case-insensitive **substring** of the name or the SIRET. Not full-text
    /// search: no stemming, no ranking, no word boundaries. The value is
    /// trimmed, and `%`, `_` and `\` match themselves. Empty after trimming
    /// means no filter, which is what a cleared search box means.
    pub q: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/host-companies",
    operation_id = "listCursusHostCompanies",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListHostCompaniesQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "A page of host companies narrowed by archive state and `q`; `total` counts the filtered rows", body = inline(DataEnvelope<Vec<HostCompanyResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: HostCompanyCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListHostCompaniesQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<HostCompanyResponse>, ApiError> {
    require_view_cursus(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let current_page = pagination.page();
    let (items, total) = state
        .usecase
        .list_host_companies(
            path.organization_id,
            HostCompanyFilter {
                archived: query.archived.unwrap_or_default(),
                search: SearchPattern::parse(query.q.as_deref()),
            },
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<HostCompanyResponse> =
        items.into_iter().map(HostCompanyResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, current_page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
