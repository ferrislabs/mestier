use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::DraftCommand;

use crate::{paths::EntrySendPath, response::TicketEntryResponse, ticket::require_manager};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/support/entries/{entry_id}/send",
    operation_id = "sendSupportDraft",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("entry_id" = mestier_core::TicketEntryId, Path, description = "Entry identifier"),
    ),
    responses(
        (status = 200, description = "The draft, now a reply visible to the client and authored by the sender", body = inline(DataEnvelope<TicketEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Entry not found in the organization"),
        (status = 409, description = "The entry is not a draft"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    EntrySendPath {
        organization_id,
        entry_id,
    }: EntrySendPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<TicketEntryResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;

    let entry = state
        .usecase
        .send_support_draft(DraftCommand {
            actor,
            organization_id,
            entry_id,
        })
        .await?;

    Ok(Response::OK(entry.into()))
}
