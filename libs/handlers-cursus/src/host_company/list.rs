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

use crate::{paths::HostCompanyCollectionPath, require_view_cursus, response::HostCompanyResponse};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListHostCompaniesQuery {
    #[param(example = false)]
    pub archived: Option<bool>,
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
        (status = 200, description = "A page of host companies; archived rows only with `archived=true`", body = inline(DataEnvelope<Vec<HostCompanyResponse>>)),
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
            query.archived.unwrap_or(false),
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
