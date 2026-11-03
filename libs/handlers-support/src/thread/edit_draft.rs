use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::EditDraftCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::EntryPath, response::TicketEntryResponse, ticket::require_manager};

#[derive(Debug, Deserialize, ToSchema)]
pub struct EditSupportDraftRequest {
    #[schema(example = "We are on it, a technician comes on Monday")]
    pub body: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/support/entries/{entry_id}",
    operation_id = "editSupportDraft",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("entry_id" = mestier_core::TicketEntryId, Path, description = "Entry identifier"),
    ),
    request_body = EditSupportDraftRequest,
    responses(
        (status = 200, description = "The draft with its new body", body = inline(DataEnvelope<TicketEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Entry not found in the organization"),
        (status = 409, description = "The entry is not a draft, or the body is blank"),
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
    Json(payload): Json<EditSupportDraftRequest>,
) -> Result<Response<TicketEntryResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;

    let entry = state
        .usecase
        .edit_support_draft(EditDraftCommand {
            actor,
            organization_id,
            entry_id,
            body: payload.body,
        })
        .await?;

    Ok(Response::OK(entry.into()))
}
