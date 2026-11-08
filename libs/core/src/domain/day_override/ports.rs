use chrono::NaiveDate;
use common::CoreError;

use super::{DayOverride, PlanningPeriod};
use crate::{MemberId, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DayOverrideRepository: Send {
    fn list(
        &mut self,
        organization_id: OrganizationId,
        period: PlanningPeriod,
        member_id: Option<MemberId>,
    ) -> impl Future<Output = Result<Vec<DayOverride>, CoreError>> + Send;

    fn upsert(
        &mut self,
        day_override: &DayOverride,
    ) -> impl Future<Output = Result<DayOverride, CoreError>> + Send;

    fn delete(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        date: NaiveDate,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
