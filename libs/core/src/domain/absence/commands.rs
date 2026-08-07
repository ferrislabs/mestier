use chrono::{DateTime, Utc};

use crate::{
    EmployeeId, OrganizationId,
    domain::absence::{AbsenceKind, EmployeeAbsenceId},
};

#[derive(Debug, Clone)]
pub struct CreateAbsenceCommand {
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub kind: AbsenceKind,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub note: Option<String>,
}

/// Carries a `PATCH`: every field is optional and only the ones present are
/// applied. `note` is itself nullable, so it needs the double option —
/// `None` means "leave unchanged", `Some(None)` means "clear it",
/// `Some(Some(value))` means "set it". `employee_id` is not patchable: an
/// absence's owner is fixed at creation, the same way a work order's
/// customer/context are — only its schedule, kind and note can move.
#[derive(Debug, Clone)]
pub struct PatchAbsenceCommand {
    pub id: EmployeeAbsenceId,
    pub kind: Option<AbsenceKind>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: Option<bool>,
    pub note: Option<Option<String>>,
}

impl PatchAbsenceCommand {
    /// A no-op patch targeting `id`: every field left unset. Tests and
    /// callers flip on only the fields they mean to change.
    pub fn new(id: EmployeeAbsenceId) -> Self {
        Self {
            id,
            kind: None,
            starts_at: None,
            ends_at: None,
            all_day: None,
            note: None,
        }
    }
}
