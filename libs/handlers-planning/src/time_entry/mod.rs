//! Field clocking: start/stop time entries on work orders, attach site photos,
//! and resolve the caller's active entry.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState};
use mestier_core::{
    EmployeeId, OrganizationId, TimeEntry, TimeEntryId, UserId, WorkOrderId,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::require_org_membership;

pub mod attach_photos;
pub mod get_active;
pub mod start;
pub mod stop;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_post(start::handler)
        .typed_post(stop::handler)
        .typed_post(attach_photos::handler)
        .typed_get(get_active::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}/time-entries/start"
)]
pub struct StartTimeEntryPath {
    pub organization_id: OrganizationId,
    pub work_order_id: WorkOrderId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/time-entries/{time_entry_id}/stop")]
pub struct StopTimeEntryPath {
    pub organization_id: OrganizationId,
    pub time_entry_id: TimeEntryId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/time-entries/{time_entry_id}/photos")]
pub struct TimeEntryPhotosPath {
    pub organization_id: OrganizationId,
    pub time_entry_id: TimeEntryId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/time-entries/active")]
pub struct ActiveTimeEntryPath {
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TimeEntryResponse {
    pub id: TimeEntryId,
    pub organization_id: OrganizationId,
    pub work_order_id: WorkOrderId,
    pub employee_id: EmployeeId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub photos_before: Vec<String>,
    pub photos_during: Vec<String>,
    pub photos_after: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TimeEntry> for TimeEntryResponse {
    fn from(value: TimeEntry) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            work_order_id: value.work_order_id,
            employee_id: value.employee_id,
            started_at: value.started_at,
            ended_at: value.ended_at,
            photos_before: value.photos_before,
            photos_during: value.photos_during,
            photos_after: value.photos_after,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// Resolves the authenticated user for an organization and returns their
/// linked employee when present.
pub(crate) async fn resolve_caller_user(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<(UserId, Option<EmployeeId>), ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    let employee = state
        .usecase
        .find_employee_by_user_id(organization_id, user.id)
        .await?;

    Ok((user.id, employee.map(|e| e.id)))
}

/// Ensures the caller may act on `employee_id`: either they are that employee
/// (linked via `user_id`), or they passed an explicit org-scoped target that
/// belongs to the organization (manager/colleague clocking is allowed for MVP
/// as long as membership holds — ownership still blocks cross-tenant).
pub(crate) async fn require_employee_access(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    employee_id: EmployeeId,
) -> Result<(), ApiError> {
    let (_user_id, caller_employee_id) =
        resolve_caller_user(state, identity, organization_id).await?;

    let employee = state.usecase.get_employee(employee_id).await?;
    if employee.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    // Prefer ownership when the caller is an employee: they may only clock
    // themselves. Org members without an employee record (office/admin) may
    // act on any employee in the organization.
    if let Some(caller_employee_id) = caller_employee_id {
        if caller_employee_id != employee_id {
            return Err(ApiError::Forbidden);
        }
    }

    Ok(())
}

pub(crate) async fn require_time_entry(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    time_entry_id: TimeEntryId,
) -> Result<TimeEntry, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let time_entry = state.usecase.get_time_entry(time_entry_id).await?;
    if time_entry.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    require_employee_access(state, identity, organization_id, time_entry.employee_id).await?;

    Ok(time_entry)
}
