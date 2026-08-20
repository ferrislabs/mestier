use chrono::{DateTime, Utc};

use crate::{EmployeeId, OrganizationId, TaskId, TimeEntryId, TimeEntryPhotoPhase, Tz};

/// Clock on to a task.
///
/// `at` is supplied rather than read from the clock inside the service, so a
/// test can pin it and so a later offline client can replay a stamp taken when
/// the employee actually pressed the button.
#[derive(Debug, Clone, Copy)]
pub struct StartTimeEntryCommand {
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub employee_id: EmployeeId,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct StopTimeEntryCommand {
    pub id: TimeEntryId,
    pub at: DateTime<Utc>,
    /// Decides which calendar day `at` falls in, which is what tells a normal
    /// clock-off from a forgotten one.
    pub timezone: Tz,
}

/// Closes a stretch the employee forgot, at the time they now declare.
#[derive(Debug, Clone, Copy)]
pub struct RecoverTimeEntryCommand {
    pub id: TimeEntryId,
    /// When the employee says they actually finished.
    pub ended_at: DateTime<Utc>,
    /// Now, against which staleness is judged. Passed rather than read so a
    /// test can pin it.
    pub now: DateTime<Utc>,
    pub timezone: Tz,
}

#[derive(Debug, Clone)]
pub struct AttachTimeEntryPhotoCommand {
    pub time_entry_id: TimeEntryId,
    pub phase: TimeEntryPhotoPhase,
    /// A key returned by the file upload endpoint, never a client-chosen path.
    pub storage_key: String,
    pub at: DateTime<Utc>,
}

/// Declare the working day over, closing whatever is still running.
#[derive(Debug, Clone, Copy)]
pub struct EndDayCommand {
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    /// Chosen by the employee, who may be declaring an earlier moment than now.
    pub ended_at: DateTime<Utc>,
    /// The organization's timezone, which decides the calendar day `ended_at`
    /// falls in. Passed in because a timezone is organization configuration,
    /// and the domain must not reach for it.
    pub timezone: Tz,
}
