use chrono::{DateTime, NaiveDate, Utc};

use super::DayPart;
use crate::{
    AbsenceTypeId, MemberId, OrganizationId,
    domain::absence::{AbsenceId, AbsenceKind},
};

#[derive(Debug, Clone)]
pub enum AbsenceSchedule {
    Window {
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        all_day: bool,
    },
    Days {
        start_date: NaiveDate,
        start_part: DayPart,
        end_date: NaiveDate,
        end_part: DayPart,
    },
}

#[derive(Debug, Clone)]
pub struct CreateAbsenceCommand {
    /// Authenticated actor performing the update. Built by the handler from
    /// the request `Identity`; carries the AuthZen-shaped subject the policy
    /// engine consumes.
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    /// Only read when `type_id` is absent: the absence then takes the default
    /// type of this kind and is approved at once.
    pub kind: Option<AbsenceKind>,
    pub type_id: Option<AbsenceTypeId>,
    pub schedule: AbsenceSchedule,
    pub note: Option<String>,
    pub proof_file_key: Option<String>,
}

/// Carries a `PATCH`: every field is optional and only the ones present are
/// applied. `note` is itself nullable, so it needs the double option —
/// `None` means "leave unchanged", `Some(None)` means "clear it",
/// `Some(Some(value))` means "set it". `member_id` is not patchable: an
/// absence's owner is fixed at creation, the same way a work order's
/// customer/context are — only its schedule, kind and note can move.
#[derive(Debug, Clone)]
pub struct PatchAbsenceCommand {
    /// Authenticated actor performing the update. Built by the handler from
    /// the request `Identity`; carries the AuthZen-shaped subject the policy
    /// engine consumes.
    pub actor: authz::Subject,
    pub id: AbsenceId,
    pub kind: Option<AbsenceKind>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: Option<bool>,
    pub note: Option<Option<String>>,
    pub proof_file_key: Option<Option<String>>,
}

impl PatchAbsenceCommand {
    /// A no-op patch targeting `id`: every field left unset. Tests and
    /// callers flip on only the fields they mean to change.
    pub fn new(id: AbsenceId, actor: authz::Subject) -> Self {
        Self {
            actor,
            id,
            kind: None,
            starts_at: None,
            ends_at: None,
            all_day: None,
            note: None,
            proof_file_key: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecideAbsenceCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: AbsenceId,
}

#[derive(Debug, Clone)]
pub struct RejectAbsenceCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: AbsenceId,
    pub comment: String,
}

#[derive(Debug, Clone)]
pub struct ListAbsencesCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub member_id: Option<MemberId>,
    pub limit: u64,
    pub offset: u64,
}
