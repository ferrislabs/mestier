use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    DayLog, DayLogId, EmployeeId, OrganizationId, TaskId, TimeEntry, TimeEntryId, TimeEntryPhoto,
    TimeEntryPhotoId, TimeEntryPhotoPhase,
};

#[derive(Debug, Clone)]
pub struct TimeEntryRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub task_id: Uuid,
    pub employee_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TimeEntryRow {
    /// Photos are loaded separately and grafted on, so a caller that only
    /// needs the timing does not pay for a join it will not read.
    pub fn into_time_entry(self, photos: Vec<TimeEntryPhoto>) -> TimeEntry {
        TimeEntry {
            id: TimeEntryId(self.id),
            organization_id: OrganizationId(self.org_id),
            task_id: TaskId(self.task_id),
            employee_id: EmployeeId(self.employee_id),
            started_at: self.started_at,
            ended_at: self.ended_at,
            photos,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeEntryPhotoRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub time_entry_id: Uuid,
    pub phase: String,
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
}

impl TimeEntryPhotoRow {
    pub fn into_photo(self) -> Result<TimeEntryPhoto, CoreError> {
        let phase = TimeEntryPhotoPhase::from_str(&self.phase).map_err(|e| {
            CoreError::Internal(format!("invalid time entry photo phase in database: {e}"))
        })?;

        Ok(TimeEntryPhoto {
            id: TimeEntryPhotoId(self.id),
            organization_id: OrganizationId(self.org_id),
            time_entry_id: TimeEntryId(self.time_entry_id),
            phase,
            storage_key: self.storage_key,
            created_at: self.created_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DayLogRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub employee_id: Uuid,
    pub work_date: NaiveDate,
    pub ended_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl DayLogRow {
    pub fn into_day_log(self) -> DayLog {
        DayLog {
            id: DayLogId(self.id),
            organization_id: OrganizationId(self.org_id),
            employee_id: EmployeeId(self.employee_id),
            work_date: self.work_date,
            ended_at: self.ended_at,
            created_at: self.created_at,
        }
    }
}
