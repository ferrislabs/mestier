use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ListThreadCommand;

use crate::{paths::TicketEntriesPath, require_view_support, response::TicketEntryResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries",
    operation_id = "listSupportTicketEntries",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
    ),
    responses(
        (status = 200, description = "Every entry of the thread in creation order: client messages, replies, internal notes and drafts", body = inline(DataEnvelope<Vec<TicketEntryResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Ticket not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TicketEntriesPath {
        organization_id,
        ticket_id,
    }: TicketEntriesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<TicketEntryResponse>>, ApiError> {
    require_view_support(&state, &identity, organization_id).await?;

    let entries = state
        .usecase
        .list_support_thread(ListThreadCommand {
            organization_id,
            ticket_id,
            client_visible_only: false,
        })
        .await?;

    Ok(Response::OK(
        entries.into_iter().map(TicketEntryResponse::from).collect(),
    ))
}
