use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::DraftCommand;

use crate::{paths::EntryPath, response::EmptyResponse, ticket::require_manager};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/support/entries/{entry_id}",
    operation_id = "deleteSupportDraft",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("entry_id" = mestier_core::TicketEntryId, Path, description = "Entry identifier"),
    ),
    responses(
        (status = 204, description = "Draft deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Entry not found in the organization"),
        (status = 409, description = "The entry is not a draft"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    EntryPath {
        organization_id,
        entry_id,
    }: EntryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;

    state
        .usecase
        .delete_support_draft(DraftCommand {
            actor,
            organization_id,
            entry_id,
        })
        .await?;

    Ok(Response::NoContent)
}
