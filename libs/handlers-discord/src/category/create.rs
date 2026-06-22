use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::CreateCategoryCommand;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::OrgCategoriesPath, require_permission, response::CategoryResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub position: i32,
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/organizations/{organization_id}/categories",
    operation_id = "createCategory",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateCategoryRequest,
    responses(
        (status = 201, description = "Category created", body = inline(DataEnvelope<CategoryResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgCategoriesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<Response<CategoryResponse>, ApiError> {
    require_permission(&state, &identity, path.organization_id, "category.manage").await?;

    if payload.name.trim().is_empty() {
        return Err(ApiError::Validation(
            "category name must not be blank".into(),
        ));
    }

    let category = state
        .usecase
        .create_category(CreateCategoryCommand {
            organization_id: path.organization_id,
            name: payload.name,
            position: payload.position,
        })
        .await?;

    Ok(Response::Created(category.into()))
}
