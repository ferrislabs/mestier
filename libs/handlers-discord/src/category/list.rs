use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::OrgCategoriesPath, require_org_membership, response::CategoryResponse};

#[utoipa::path(
    get,
    path = "/api/v1/chat/organizations/{organization_id}/categories",
    operation_id = "listCategories",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "List of categories", body = inline(DataEnvelope<Vec<CategoryResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgCategoriesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<CategoryResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let categories = state.usecase.list_categories(path.organization_id).await?;
    let items: Vec<CategoryResponse> = categories.into_iter().map(CategoryResponse::from).collect();
    Ok(Response::OK(items))
}
