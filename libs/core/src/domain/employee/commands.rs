use crate::{MemberId, OrganizationId};

/// Attaches a contractual profile to a member, or updates the one already
/// there. One command for both because the caller's intent is the same —
/// "this member's contract says X" — and the distinction between a first and a
/// later call is an implementation detail of the repository, not of the domain.
#[derive(Debug, Clone)]
pub struct UpsertEmployeeProfileCommand {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    /// `None` means the rate is not set yet; `Some(0)` means genuinely free.
    /// Ignored when `is_salaried` is set — see [`super::service::validate_rate`].
    pub hourly_rate_cents: Option<i32>,
    pub is_salaried: bool,
    /// Ignored unless `is_salaried` is set — the two cost bases are exclusive.
    pub monthly_cost_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
}

/// Detaches the contractual profile from a member. The seat, its assignments,
/// its absences and its work slots all survive — only the contract goes.
#[derive(Debug, Clone)]
pub struct RemoveEmployeeProfileCommand {
    pub member_id: MemberId,
}
