use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use super::dto::CollectionResponse;
use crate::{paths::CollectionPath, require_view_automation};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}",
    operation_id = "getCollection",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("collection_id" = uuid::Uuid, Path, description = "Collection identifier"),
    ),
    responses(
        (status = 200, description = "The whole collection with its operations, schemas and version history, newest first", body = inline(DataEnvelope<CollectionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Collection not found in this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CollectionPath {
        organization_id,
        collection_id,
    }: CollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<CollectionResponse>, ApiError> {
    require_view_automation(&state, &identity, organization_id).await?;

    let detail = state
        .usecase
        .find_collection_detail(organization_id, collection_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Response::OK(CollectionResponse::from(detail)))
}
