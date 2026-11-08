use authz::Subject;
use chrono::NaiveDate;

use super::{ExtraPeriodId, ExtraStatus};
use crate::{LunchInput, MemberId, OrganizationId, SurchargeId};

#[derive(Debug, Clone)]
pub struct ExtraPeriodDraft {
    pub member_id: MemberId,
    pub date: NaiveDate,
    pub start: String,
    pub end: String,
    pub lunch: Option<LunchInput>,
    pub reason: String,
    pub surcharge_id: Option<SurchargeId>,
    pub rate_bp: Option<i32>,
    pub approve: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ExtraPeriodFilter {
    pub member_id: Option<MemberId>,
    pub status: Option<ExtraStatus>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct ListExtraPeriodsCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub filter: ExtraPeriodFilter,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub struct CreateExtraPeriodCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub draft: ExtraPeriodDraft,
}

#[derive(Debug, Clone)]
pub struct DecideExtraPeriodCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: ExtraPeriodId,
}

#[derive(Debug, Clone)]
pub struct RejectExtraPeriodCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: ExtraPeriodId,
    pub comment: String,
}
