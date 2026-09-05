use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{CustomerId, QuoteId};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{project::ProjectsPath, require_org_membership, response::ProjectResponse};

/// `?customer_id=` narrows to one client's projects. `?quote_id=` narrows to
/// the (at most one) project a given quote turned into. `?include_archived=true`
/// brings back retired ones, which a report needs and a picker does not — the
/// default is the picker's.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListProjectsQuery {
    #[serde(default)]
    pub customer_id: Option<CustomerId>,
    #[serde(default)]
    pub quote_id: Option<QuoteId>,
    #[serde(default)]
    pub include_archived: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/projects",
    operation_id = "listProjects",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
        ListProjectsQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of projects", body = inline(DataEnvelope<Vec<ProjectResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ProjectsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
    Query(filter): Query<ListProjectsQuery>,
) -> Result<Response<ProjectResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let (projects, total) = state
        .usecase
        .list_projects(
            path.organization_id,
            filter.customer_id,
            filter.quote_id,
            filter.include_archived,
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<ProjectResponse> = projects.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
