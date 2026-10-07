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
    doe_model::{DoeModelsPath, response::DoeModelResponse},
    require_org_membership,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListDoeModelsQuery {
    #[serde(default)]
    pub include_archived: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/doe-models",
    operation_id = "listDoeModels",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
        ListDoeModelsQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of DOE models", body = inline(DataEnvelope<Vec<DoeModelResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DoeModelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
    Query(filter): Query<ListDoeModelsQuery>,
) -> Result<Response<DoeModelResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let (models, total) = state
        .usecase
        .list_doe_models(
            path.organization_id,
            filter.include_archived,
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<DoeModelResponse> = models.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
