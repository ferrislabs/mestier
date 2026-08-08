use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};

use crate::{EmptyResponse, paths::CategoryPath, require_permission};

#[utoipa::path(
    delete,
    path = "/api/v1/chat/categories/{category_id}",
    operation_id = "deleteCategory",
    tag = super::super::TAG,
    params(("category_id" = discord::CategoryId, Path, description = "Category identifier")),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
        (status = 404, description = "Category not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CategoryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let existing = state.usecase.get_category(path.category_id).await?;
    require_permission(
        &state,
        &identity,
        existing.organization_id,
        "category.manage",
    )
    .await?;
    let actor = resolve_user_id(&state, &identity).await?;
    state
        .usecase
        .acting_as(actor)
        .delete_category(path.category_id)
        .await?;
    Ok(Response::NoContent)
}
