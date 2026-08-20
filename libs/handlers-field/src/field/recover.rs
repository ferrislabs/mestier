use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::TimeEntryId;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::FieldRecoverPath, resolve_field_actor, response::TimeEntryResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecoverTimeEntryRequest {
    /// When the employee says they actually finished. On an earlier day than
    /// today, which is the whole point of this route.
    pub ended_at: DateTime<Utc>,
}

/// Closes a stretch the employee forgot to clock off.
///
/// The one route allowed to accept an end on a day before today, because it is
/// the one where somebody is stating a fact rather than pressing a button as it
/// happens. The entry is marked as declared after the fact, so the profitability
/// report can tell a recollection from a measurement.
///
/// Refuses a stretch from today: that one is stopped, not recovered.
#[utoipa::path(
    post,
    path = "/api/v1/field/time-entries/{time_entry_id}/recover",
    operation_id = "recoverTimeEntry",
    tag = crate::TAG,
    params(("time_entry_id" = TimeEntryId, Path, description = "Time entry identifier")),
    request_body = RecoverTimeEntryRequest,
    responses(
        (status = 200, description = "Closed at the declared time", body = inline(DataEnvelope<TimeEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No such entry for this caller"),
        (status = 409, description = "Already closed, from today, or ending before it started"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldRecoverPath { time_entry_id }: FieldRecoverPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(body): Json<RecoverTimeEntryRequest>,
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

    let recovered = state
        .usecase
        .recover_forgotten_time_entry(
            entry.organization_id,
            time_entry_id,
            body.ended_at,
            Utc::now(),
        )
        .await?;

    Ok(Response::OK(recovered.into()))
}
