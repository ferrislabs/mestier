use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::UpdateCategoryCommand;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::CategoryPath, require_permission, response::CategoryResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCategoryRequest {
    pub name: String,
    pub position: i32,
}

#[utoipa::path(
    patch,
    path = "/api/v1/chat/categories/{category_id}",
    operation_id = "updateCategory",
    tag = super::super::TAG,
    params(("category_id" = discord::CategoryId, Path, description = "Category identifier")),
    request_body = UpdateCategoryRequest,
    responses(
        (status = 200, description = "Category updated", body = inline(DataEnvelope<CategoryResponse>)),
        (status = 400, description = "Validation failed"),
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
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Response<CategoryResponse>, ApiError> {
    // Fetch the category first to obtain organization_id for the permission check.
    let existing = state.usecase.get_category(path.category_id).await?;

    require_permission(
        &state,
        &identity,
        existing.organization_id,
        "category.manage",
    )
    .await?;
    let actor = resolve_user_id(&state, &identity).await?;

    if payload.name.trim().is_empty() {
        return Err(ApiError::Validation(
            "category name must not be blank".into(),
        ));
    }

    let updated = state
        .usecase
        .acting_as(actor)
        .update_category(UpdateCategoryCommand {
            id: path.category_id,
            name: payload.name,
            position: payload.position,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
