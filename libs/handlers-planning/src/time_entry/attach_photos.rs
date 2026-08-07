use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{AttachTimeEntryPhotosCommand, TimeEntryPhotoPhase};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::time_entry::{TimeEntryPhotosPath, TimeEntryResponse, require_time_entry};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AttachTimeEntryPhotosRequest {
    pub phase: TimeEntryPhotoPhase,
    /// Opaque keys previously returned by `POST /api/v1/files`.
    pub photo_keys: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/time-entries/{time_entry_id}/photos",
    operation_id = "attachTimeEntryPhotos",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("time_entry_id" = mestier_core::TimeEntryId, Path, description = "Time entry identifier"),
    ),
    request_body = AttachTimeEntryPhotosRequest,
    responses(
        (status = 200, description = "Photos attached", body = inline(DataEnvelope<TimeEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Invalid photo keys"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TimeEntryPhotosPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AttachTimeEntryPhotosRequest>,
) -> Result<Response<TimeEntryResponse>, ApiError> {
    require_time_entry(
        &state,
        &identity,
        path.organization_id,
        path.time_entry_id,
    )
    .await?;

    let time_entry = state
        .usecase
        .attach_time_entry_photos(AttachTimeEntryPhotosCommand {
            id: path.time_entry_id,
            phase: payload.phase,
            photo_keys: payload.photo_keys,
        })
        .await?;

    Ok(Response::OK(time_entry.into()))
}
