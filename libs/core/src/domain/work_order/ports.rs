use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, WorkOrder, WorkOrderId};

#[cfg_attr(test, mockall::automock)]
pub trait WorkOrderRepository: Send {
    fn insert(
        &mut self,
        work_order: &WorkOrder,
    ) -> impl Future<Output = Result<WorkOrder, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: WorkOrderId,
    ) -> impl Future<Output = Result<Option<WorkOrder>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<WorkOrder>, u64), CoreError>> + Send;

    /// Persists both the work order's own fields and its complete
    /// `assignments` / `equipment` lists. The infra adapter replaces both as
    /// a whole (physical delete then insert) rather than diffing — matching
    /// the `PATCH` contract, where `assignees` / `equipment` are never a
    /// delta.
    fn update(
        &mut self,
        work_order: &WorkOrder,
    ) -> impl Future<Output = Result<WorkOrder, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: WorkOrderId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
