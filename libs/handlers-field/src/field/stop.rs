use auth::Identity;
use axum::{Extension, extract::State};
use chrono::Utc;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::TimeEntryId;

use crate::{paths::FieldStopPath, resolve_field_actor, response::TimeEntryResponse};

/// Clocks the caller off the job they are on.
///
/// Answers 409 for a stretch begun on an earlier day: closing it now would
/// record hours nobody worked, so the app asks the employee for the real time
/// and calls the recover route instead.
///
/// Refuses an entry that belongs to somebody else. The id is in the path, so
/// unlike the other routes this one has to check ownership rather than derive
/// it, and a 404 rather than a 403 keeps it from confirming the entry exists.
#[utoipa::path(
    post,
    path = "/api/v1/field/time-entries/{time_entry_id}/stop",
    operation_id = "stopTimeEntry",
    tag = crate::TAG,
    params(("time_entry_id" = TimeEntryId, Path, description = "Time entry identifier")),
    responses(
        (status = 200, description = "Clocked off", body = inline(DataEnvelope<TimeEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No such entry for this caller"),
        (status = 409, description = "Entry already closed, or begun on an earlier day and needing its end declared"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldStopPath { time_entry_id }: FieldStopPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<TimeEntryResponse>, ApiError> {
    let entry = state
        .usecase
        .get_time_entry(time_entry_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let actor = resolve_field_actor(&state, &identity, entry.organization_id).await?;
    if entry.employee_id != actor.employee_id {
        return Err(ApiError::NotFound);
    }

    let stopped = state
        .usecase
        .stop_time_entry(entry.organization_id, time_entry_id, Utc::now())
        .await?;

    Ok(Response::OK(stopped.into()))
}
