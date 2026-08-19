use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    CustomerContextId, CustomerId, DayLog, DayLogId, EmployeeId, OrganizationId, Task, TaskId,
    TaskStatus, TimeEntry, TimeEntryId, TimeEntryPhoto, TimeEntryPhotoId, TimeEntryPhotoPhase,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TimeEntryPhotoResponse {
    pub id: TimeEntryPhotoId,
    pub time_entry_id: TimeEntryId,
    pub phase: TimeEntryPhotoPhase,
    /// The storage key. Turn it into something displayable with
    /// `GET /api/v1/files/url`, which the field app already uses for quotes.
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
}

impl From<TimeEntryPhoto> for TimeEntryPhotoResponse {
    fn from(value: TimeEntryPhoto) -> Self {
        Self {
            id: value.id,
            time_entry_id: value.time_entry_id,
            phase: value.phase,
            storage_key: value.storage_key,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TimeEntryResponse {
    pub id: TimeEntryId,
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub employee_id: EmployeeId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    /// Absent while the entry is still running, so a client never has to
    /// subtract two timestamps and disagree with the server about the total.
    pub worked_minutes: Option<i64>,
    pub photos: Vec<TimeEntryPhotoResponse>,
}

impl From<TimeEntry> for TimeEntryResponse {
    fn from(value: TimeEntry) -> Self {
        let worked_minutes = value.worked_minutes();
        Self {
            id: value.id,
            organization_id: value.organization_id,
            task_id: value.task_id,
            employee_id: value.employee_id,
            started_at: value.started_at,
            ended_at: value.ended_at,
            worked_minutes,
            photos: value.photos.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DayLogResponse {
    pub id: DayLogId,
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub work_date: NaiveDate,
    pub ended_at: DateTime<Utc>,
}

impl From<DayLog> for DayLogResponse {
    fn from(value: DayLog) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            employee_id: value.employee_id,
            work_date: value.work_date,
            ended_at: value.ended_at,
        }
    }
}

/// A job as the field app needs it: what to do, when, and for whom.
///
/// Carries ids for the customer and the site rather than their addresses. The
/// app already resolves those through the customer endpoints, and inlining
/// them here would duplicate a read that is cached anyway.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct FieldTaskResponse {
    pub id: TaskId,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub status: TaskStatus,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
}

impl From<Task> for FieldTaskResponse {
    fn from(value: Task) -> Self {
        Self {
            id: value.id,
            title: value.title,
            description: value.description,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            all_day: value.all_day,
            status: value.status,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
        }
    }
}
