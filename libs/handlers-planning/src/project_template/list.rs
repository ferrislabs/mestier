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

use crate::{
    project_template::ProjectTemplatesPath, require_org_membership,
    response::ProjectTemplateResponse,
};

/// `?include_archived=true` brings back retired templates, which a report
/// needs and a picker does not — the default is the picker's.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListProjectTemplatesQuery {
    #[serde(default)]
    pub include_archived: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/project-templates",
    operation_id = "listProjectTemplates",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
        ListProjectTemplatesQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of templates", body = inline(DataEnvelope<Vec<ProjectTemplateResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ProjectTemplatesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
    Query(filter): Query<ListProjectTemplatesQuery>,
) -> Result<Response<ProjectTemplateResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let (templates, total) = state
        .usecase
        .list_project_templates(
            path.organization_id,
            filter.include_archived,
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<ProjectTemplateResponse> = templates.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
