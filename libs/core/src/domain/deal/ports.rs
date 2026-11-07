use chrono::{DateTime, Utc};
use common::CoreError;

use super::{StageTotal, commands::DealFilter};
use crate::{CustomerId, Deal, DealId, LostReason, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DealRepository: Send {
    fn insert(&mut self, deal: &Deal) -> impl Future<Output = Result<Deal, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: DealId,
    ) -> impl Future<Output = Result<Option<Deal>, CoreError>> + Send;

    fn list(
        &mut self,
        filter: &DealFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Deal>, u64), CoreError>> + Send;

    fn update(&mut self, deal: &Deal) -> impl Future<Output = Result<Deal, CoreError>> + Send;

    fn delete(&mut self, id: DealId) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn lose_open_of_customer(
        &mut self,
        customer_id: CustomerId,
        reason: &LostReason,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn stage_totals(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<StageTotal>, CoreError>> + Send;

    fn closed_totals_since(
        &mut self,
        organization_id: OrganizationId,
        since: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<StageTotal>, CoreError>> + Send;
}
