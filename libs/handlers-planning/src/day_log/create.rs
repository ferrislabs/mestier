use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CloseDayCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    day_log::{DayLogResponse, EmployeeDayLogsPath},
    time_entry::require_employee_access,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CloseDayRequest {
    pub work_date: NaiveDate,
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/employees/{employee_id}/day-logs",
    operation_id = "closeDay",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("employee_id" = mestier_core::EmployeeId, Path, description = "Employee identifier"),
    ),
    request_body = CloseDayRequest,
    responses(
        (status = 201, description = "Day closed", body = inline(DataEnvelope<DayLogResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Open time entry still active, or day already closed"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EmployeeDayLogsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CloseDayRequest>,
) -> Result<Response<DayLogResponse>, ApiError> {
    require_employee_access(&state, &identity, path.organization_id, path.employee_id).await?;

    let day_log = state
        .usecase
        .close_day(CloseDayCommand {
            organization_id: path.organization_id,
            employee_id: path.employee_id,
            work_date: payload.work_date,
            ended_at: payload.ended_at,
        })
        .await?;

    Ok(Response::Created(day_log.into()))
}
