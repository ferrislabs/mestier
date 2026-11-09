use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::OrgDirectoryPath, require_org_membership, response::DirectoryEntryResponse};

#[utoipa::path(
    get,
    path = "/api/v1/chat/organizations/{organization_id}/directory",
    operation_id = "listChatDirectory",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Seated members, keyed by the user id chat uses", body = inline(DataEnvelope<Vec<DirectoryEntryResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgDirectoryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<DirectoryEntryResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let entries = state
        .usecase
        .list_chat_directory(path.organization_id)
        .await?;
    Ok(Response::OK(
        entries
            .into_iter()
            .map(DirectoryEntryResponse::from)
            .collect(),
    ))
}
