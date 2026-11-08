use authz::Subject;
use chrono::NaiveDate;

use super::{AdjustmentKind, LeaveAdjustmentId};
use crate::{LeaveAccountId, MemberId, OrganizationId};

#[derive(Debug, Clone)]
pub struct LeaveAdjustmentDraft {
    pub member_id: MemberId,
    pub account_id: LeaveAccountId,
    pub date: NaiveDate,
    pub days: f64,
    pub kind: AdjustmentKind,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ListLeaveAdjustmentsCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: Option<MemberId>,
    pub account_id: Option<LeaveAccountId>,
}

#[derive(Debug, Clone)]
pub struct CreateLeaveAdjustmentCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub draft: LeaveAdjustmentDraft,
}

#[derive(Debug, Clone)]
pub struct DeleteLeaveAdjustmentCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: LeaveAdjustmentId,
}
