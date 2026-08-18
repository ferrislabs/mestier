use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::Utc;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{OrganizationId, StartTimeEntryCommand, TaskId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::FieldTimeEntriesPath, resolve_field_actor, response::TimeEntryResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartTimeEntryRequest {
    pub task_id: TaskId,
}

/// Clocks the caller on to a job.
///
/// Answers 409 when they are already clocked on somewhere, with the open job
/// named in the message, so the app can offer to close it first.
///
/// The employee is the caller, always: nothing in the body can point this at
/// somebody else.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/field/time-entries",
    operation_id = "startTimeEntry",
    tag = crate::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    request_body = StartTimeEntryRequest,
    responses(
        (status = 201, description = "Clocked on", body = inline(DataEnvelope<TimeEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Caller is not an employee of this organization"),
        (status = 409, description = "Already clocked on to another job"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldTimeEntriesPath { organization_id }: FieldTimeEntriesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(body): Json<StartTimeEntryRequest>,
) -> Result<Response<TimeEntryResponse>, ApiError> {
    let actor = resolve_field_actor(&state, &identity, organization_id).await?;

    let entry = state
        .usecase
        .start_time_entry(StartTimeEntryCommand {
            organization_id,
            task_id: body.task_id,
            employee_id: actor.employee_id,
            // Server-stamped. A client clock is trusted only once the offline
            // mode exists to justify it, and it does not yet.
            at: Utc::now(),
        })
        .await?;

    Ok(Response::Created(entry.into()))
}
