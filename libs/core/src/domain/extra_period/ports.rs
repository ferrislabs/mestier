use common::CoreError;

use super::{ExtraPeriod, ExtraPeriodId, commands::ExtraPeriodFilter};
use crate::OrganizationId;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ExtraPeriodRepository: Send {
    fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &ExtraPeriodFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<ExtraPeriod>, u64), CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: ExtraPeriodId,
    ) -> impl Future<Output = Result<Option<ExtraPeriod>, CoreError>> + Send;

    fn insert(
        &mut self,
        period: &ExtraPeriod,
    ) -> impl Future<Output = Result<ExtraPeriod, CoreError>> + Send;

    fn update(
        &mut self,
        period: &ExtraPeriod,
    ) -> impl Future<Output = Result<ExtraPeriod, CoreError>> + Send;
}
