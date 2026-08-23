use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    AssignmentReport, AssignmentReportId, AssignmentReportResolution, CustomerContextId,
    CustomerId, DayLog, DayLogId, EmployeeId, MemberId, OrganizationId, Task, TaskAssignmentId,
    TaskId, TaskStatus, TimeEntry, TimeEntryId, TimeEntryPhoto, TimeEntryPhotoId,
    TimeEntryPhotoPhase,
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

/// What the field app needs on load: what the caller is clocked on to, and
/// whether they have already declared the day over.
///
/// Bundled into one response because both answer the same question — "where
/// do things stand right now" — and the app would otherwise need a second
/// round trip just to learn the day is already closed, which is exactly the
/// trip a bad connection on a job site can lose.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct FieldCurrentResponse {
    pub running: Option<TimeEntryResponse>,
    pub day_ended_at: Option<DateTime<Utc>>,
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

/// The worker's report of an assignment's actual duration, and whatever a
/// manager has decided about it so far — never a rate or a cost. See
/// `field-day-feature.tsx`'s "minutes, not money" rule: this screen has
/// nothing to say about anyone's salary.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AssignmentReportResponse {
    pub id: AssignmentReportId,
    pub organization_id: OrganizationId,
    pub task_assignment_id: TaskAssignmentId,
    pub reported_minutes: u32,
    pub comment: Option<String>,
    pub reported_by: MemberId,
    pub resolution: AssignmentReportResolution,
    pub resolved_by: Option<MemberId>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AssignmentReport> for AssignmentReportResponse {
    fn from(value: AssignmentReport) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            task_assignment_id: value.task_assignment_id,
            reported_minutes: value.reported_minutes,
            comment: value.comment,
            reported_by: value.reported_by,
            resolution: value.resolution,
            resolved_by: value.resolved_by,
            resolved_at: value.resolved_at,
            resolution_note: value.resolution_note,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
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
