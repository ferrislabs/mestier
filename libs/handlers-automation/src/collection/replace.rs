use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::ReplaceCollectionCommand;

use super::dto::{CollectionRequest, CollectionResponse};
use crate::{error::AutomationError, paths::CollectionPath, require_manage_automation};

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}",
    operation_id = "replaceCollection",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("collection_id" = uuid::Uuid, Path, description = "Collection identifier"),
    ),
    request_body = CollectionRequest,
    responses(
        (status = 200, description = "Collection replaced, id and slug kept; the previous version is appended to the history in the same transaction and the answer carries the updated history, newest first", body = inline(DataEnvelope<CollectionResponse>)),
        (status = 400, description = "The model is invalid"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Collection not found in this organization"),
        (status = 413, description = "E_COLLECTION_TOO_LARGE: the model holds more than 2 MiB"),
        (status = 422, description = "The body is not a collection model"),
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
    Json(payload): Json<CollectionRequest>,
) -> Result<Response<CollectionResponse>, AutomationError> {
    require_manage_automation(&state, &identity, organization_id).await?;

    let draft = payload.into_draft()?;
    let replaced = state
        .usecase
        .replace_collection(ReplaceCollectionCommand {
            org_id: organization_id,
            id: collection_id,
            draft,
        })
        .await?;

    Ok(Response::OK(CollectionResponse::from(replaced)))
}
