use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::DateRange;

use crate::work_time::{
    RhythmResponse, WorkSlotResponse, WorkTimePath, WorkTimeResponse, WorkTimeWindowQuery,
};

/// The API's reading window is capped at 92 days: without a bound, the HR
/// screen of a three-year-old employee would download their entire work-slot
/// history just to show a week (see the planning module design doc,
/// invariant 6).
const MAX_WINDOW_DAYS: i64 = 92;

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/employees/{employee_id}/work-time",
    operation_id = "getWorkTime",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("employee_id" = mestier_core::EmployeeId, Path, description = "Employee identifier"),
        ("from" = chrono::NaiveDate, Query, description = "Window start (inclusive)"),
        ("to" = chrono::NaiveDate, Query, description = "Window end (inclusive)"),
    ),
    responses(
        (status = 200, description = "Rhythm versions overlapping the window and work slots inside it", body = inline(DataEnvelope<WorkTimeResponse>)),
        (status = 400, description = "Window wider than 92 days, or `to` before `from`"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkTimePath { member_id }: WorkTimePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(window): Query<WorkTimeWindowQuery>,
) -> Result<Response<WorkTimeResponse>, ApiError> {
    crate::require_member_target(&state, &identity, member_id).await?;

    let range = DateRange::new(window.from, window.to)?;
    if range.span_days() > MAX_WINDOW_DAYS {
        return Err(ApiError::BadRequest(format!(
            "work-time window must not exceed {MAX_WINDOW_DAYS} days"
        )));
    }

    let (rhythms, work_slots) = state.usecase.get_work_time(member_id, range).await?;

    Ok(Response::OK(WorkTimeResponse {
        rhythms: rhythms.into_iter().map(RhythmResponse::from).collect(),
        work_slots: work_slots.into_iter().map(WorkSlotResponse::from).collect(),
    }))
}
