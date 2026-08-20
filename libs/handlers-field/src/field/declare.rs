use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{DeclareTimeEntryCommand, OrganizationId, TaskId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::FieldDeclarePath, resolve_field_actor, response::TimeEntryResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeclareTimeEntryRequest {
    pub task_id: TaskId,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
}

/// Declares a stretch of work the caller did but never clocked live.
///
/// For the rush that leaves no time to press "Démarrer": nothing here was
/// ever open to begin with, which is what tells this apart from `recover`,
/// the route that closes a forgotten clock-off. Always marked as declared
/// after the fact, exactly like a recovered entry, so the profitability
/// report never mistakes it for a live measurement.
///
/// The employee is the caller, always: nothing in the body can point this at
/// somebody else.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/field/time-entries/declare",
    operation_id = "declareTimeEntry",
    tag = crate::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    request_body = DeclareTimeEntryRequest,
    responses(
        (status = 201, description = "Declared", body = inline(DataEnvelope<TimeEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Caller is not an employee of this organization"),
        (status = 409, description = "Ends before it started, ends in the future, or overlaps a job still running"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldDeclarePath { organization_id }: FieldDeclarePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(body): Json<DeclareTimeEntryRequest>,
) -> Result<Response<TimeEntryResponse>, ApiError> {
    let actor = resolve_field_actor(&state, &identity, organization_id).await?;

    let entry = state
        .usecase
        .declare_time_entry(DeclareTimeEntryCommand {
            organization_id,
            task_id: body.task_id,
            employee_id: actor.employee_id,
            started_at: body.started_at,
            ended_at: body.ended_at,
            // Server-stamped, same reason `start` stamps `at` itself: a
            // client clock is trusted only once the offline mode exists to
            // justify it, and it does not yet.
            now: Utc::now(),
        })
        .await?;

    Ok(Response::Created(entry.into()))
}
