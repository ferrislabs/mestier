use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::ImportCollectionCommand;

use super::{
    body::CollectionBody,
    dto::{CollectionRequest, CollectionResponse},
};
use crate::{error::AutomationError, paths::CollectionsPath, require_manage_automation};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/collections",
    operation_id = "importCollection",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CollectionRequest,
    responses(
        (status = 201, description = "Collection stored; its slug is derived from the title and suffixed -2, -3 when another collection of the organization already has it", body = inline(DataEnvelope<CollectionResponse>)),
        (status = 400, description = "The model is invalid: blank title, no operation, duplicated operation id, bad path"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 413, description = "E_COLLECTION_TOO_LARGE: the model holds more than 2 MiB"),
        (status = 422, description = "The body is not a collection model"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CollectionsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CollectionBody(payload): CollectionBody,
) -> Result<Response<CollectionResponse>, AutomationError> {
    require_manage_automation(&state, &identity, path.organization_id).await?;

    let draft = payload.into_draft()?;
    let imported = state
        .usecase
        .import_collection(ImportCollectionCommand {
            org_id: path.organization_id,
            draft,
        })
        .await?;

    Ok(Response::Created(CollectionResponse::new(
        imported,
        Vec::new(),
    )))
}
