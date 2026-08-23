use chrono::NaiveDate;

use crate::{EmployeeCostBasisId, EmployeeId, MemberId, OrganizationId};

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

/// States what an employee costs from `effective_from` onward.
///
/// Mirrors [`super::service::EmployeeCostBasisService::set_cost_basis`]'s
/// semantics, itself mirroring `ReplaceRhythmCommand`: this either opens the
/// employee's very first version, edits the open version in place (when
/// `effective_from` matches it exactly — the same version being corrected the
/// same day), or closes the open version and opens a new one (when
/// `effective_from` is later). A date before the currently open version is
/// refused rather than silently reordering history.
#[derive(Debug, Clone)]
pub struct SetEmployeeCostBasisCommand {
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub effective_from: NaiveDate,
    pub is_salaried: bool,
    /// `None` means the rate is not set yet; `Some(0)` means genuinely free.
    /// Ignored when `is_salaried` is set.
    pub hourly_rate_cents: Option<i32>,
    /// Ignored unless `is_salaried` is set — the two cost bases are exclusive.
    pub monthly_cost_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
}

/// Corrects a cost basis version that was entered wrong — the dangerous verb,
/// because unlike [`SetEmployeeCostBasisCommand`] it rewrites history on
/// purpose rather than dating a new change. Every field of the version can be
/// corrected, including its own `effective_from`/`effective_to`: a typo in
/// the date is exactly the kind of mistake this exists to fix.
#[derive(Debug, Clone)]
pub struct CorrectEmployeeCostBasisCommand {
    pub id: EmployeeCostBasisId,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub is_salaried: bool,
    /// `None` means the rate is not set yet; `Some(0)` means genuinely free.
    /// Ignored when `is_salaried` is set.
    pub hourly_rate_cents: Option<i32>,
    /// Ignored unless `is_salaried` is set — the two cost bases are exclusive.
    pub monthly_cost_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
}
