use chrono::NaiveDate;

use crate::{EmployeeId, OrganizationId};

/// One slot to persist as part of a rhythm replacement — the recurring
/// weekly model's shape, decoupled from any DB-generated id (a full replace
/// always creates fresh slot rows, see [`crate::domain::work_time::service`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RhythmSlotInput {
    pub weekday: i16,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

/// Carries `PUT .../rhythm`: replaces the rhythm version currently in
/// effect wholesale, or opens a new version if `effective_from` moves past
/// it — see `WorkTimeService::replace_rhythm`.
#[derive(Debug, Clone)]
pub struct ReplaceRhythmCommand {
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub slots: Vec<RhythmSlotInput>,
}

/// One slot to persist as part of a work-slots window replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkSlotInput {
    pub work_date: NaiveDate,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

/// Carries `PUT .../work-slots?from=&to=`: replaces the `[from, to]` window
/// wholesale.
#[derive(Debug, Clone)]
pub struct ReplaceWorkSlotsCommand {
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub slots: Vec<WorkSlotInput>,
}
