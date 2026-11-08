use authz::Subject;
use chrono::NaiveDate;

use super::{OverrideKind, PlanningPeriod};
use crate::{LunchInput, MemberId, OrganizationId};

#[derive(Debug, Clone)]
pub struct DayOverrideInput {
    pub kind: OverrideKind,
    pub start: Option<String>,
    pub end: Option<String>,
    pub lunch: Option<LunchInput>,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct ListDayOverridesCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub period: PlanningPeriod,
    pub member_id: Option<MemberId>,
}

#[derive(Debug, Clone)]
pub struct PutDayOverrideCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub date: NaiveDate,
    pub input: DayOverrideInput,
}

#[derive(Debug, Clone)]
pub struct DeleteDayOverrideCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct HrPlanningFeedCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub period: PlanningPeriod,
}
