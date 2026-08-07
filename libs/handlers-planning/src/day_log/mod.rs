//! End-of-day declaration (`day_logs`).

use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::AppState;
use mestier_core::{DayLog, DayLogId, EmployeeId, OrganizationId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod create;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new().typed_post(create::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/employees/{employee_id}/day-logs")]
pub struct EmployeeDayLogsPath {
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DayLogResponse {
    pub id: DayLogId,
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub work_date: NaiveDate,
    pub ended_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<DayLog> for DayLogResponse {
    fn from(value: DayLog) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            employee_id: value.employee_id,
            work_date: value.work_date,
            ended_at: value.ended_at,
            created_at: value.created_at,
        }
    }
}
