use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::Utc;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{AttachTimeEntryPhotoCommand, TimeEntryId, TimeEntryPhotoPhase};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::FieldPhotosPath, resolve_field_actor, response::TimeEntryPhotoResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AttachPhotoRequest {
    pub phase: TimeEntryPhotoPhase,
    /// A key returned by `POST /api/v1/files`, never a client-chosen path.
    pub storage_key: String,
}

/// Attaches a photo to one of the caller's entries.
///
/// Accepted after the entry is closed: the "after" shot is usually taken once
/// the job is stopped, and refusing it would cost half of every pair.
#[utoipa::path(
    post,
    path = "/api/v1/field/time-entries/{time_entry_id}/photos",
    operation_id = "attachTimeEntryPhoto",
    tag = crate::TAG,
    params(("time_entry_id" = TimeEntryId, Path, description = "Time entry identifier")),
    request_body = AttachPhotoRequest,
    responses(
        (status = 201, description = "Photo attached", body = inline(DataEnvelope<TimeEntryPhotoResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No such entry for this caller"),
        (status = 409, description = "Invalid storage key"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldPhotosPath { time_entry_id }: FieldPhotosPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(body): Json<AttachPhotoRequest>,
) -> Result<Response<TimeEntryPhotoResponse>, ApiError> {
    let entry = state
        .usecase
        .get_time_entry(time_entry_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let actor = resolve_field_actor(&state, &identity, entry.organization_id).await?;
    if entry.employee_id != actor.employee_id {
        return Err(ApiError::NotFound);
    }

    let photo = state
        .usecase
        .attach_time_entry_photo(AttachTimeEntryPhotoCommand {
            time_entry_id,
            phase: body.phase,
            storage_key: body.storage_key,
            at: Utc::now(),
        })
        .await?;

    Ok(Response::Created(photo.into()))
}
