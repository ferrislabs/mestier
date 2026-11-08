use authz::Subject;

use super::{AcquisitionDraft, LeaveAccountId};
use crate::OrganizationId;

#[derive(Debug, Clone)]
pub struct LeaveAccountDraft {
    pub name: String,
    pub acquisition: AcquisitionDraft,
    pub period_start_month: u8,
    pub carry_over_max: f64,
}

#[derive(Debug, Clone)]
pub struct ListLeaveAccountsCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct CreateLeaveAccountCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub draft: LeaveAccountDraft,
}

#[derive(Debug, Clone)]
pub struct UpdateLeaveAccountCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: LeaveAccountId,
    pub draft: LeaveAccountDraft,
}

#[derive(Debug, Clone)]
pub struct ArchiveLeaveAccountCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: LeaveAccountId,
    pub archived: bool,
}
