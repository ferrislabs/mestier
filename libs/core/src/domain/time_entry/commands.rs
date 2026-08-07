use chrono::{DateTime, Utc};

use crate::{
    EmployeeId, OrganizationId, WorkOrderId,
    domain::time_entry::{TimeEntryId, TimeEntryPhotoPhase},
};

#[derive(Debug, Clone)]
pub struct StartTimeEntryCommand {
    pub organization_id: OrganizationId,
    pub work_order_id: WorkOrderId,
    pub employee_id: EmployeeId,
    /// Optional photos taken before starting work on site.
    pub photos_before: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StopTimeEntryCommand {
    pub id: TimeEntryId,
    pub ended_at: Option<DateTime<Utc>>,
    /// Optional photos taken when leaving the site.
    pub photos_after: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AttachTimeEntryPhotosCommand {
    pub id: TimeEntryId,
    pub phase: TimeEntryPhotoPhase,
    pub photo_keys: Vec<String>,
}
