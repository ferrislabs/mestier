use common::CoreError;

use super::{LeaveAdjustment, LeaveAdjustmentId};
use crate::{LeaveAccountId, MemberId, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait LeaveAdjustmentRepository: Send {
    fn list(
        &mut self,
        organization_id: OrganizationId,
        member_id: Option<MemberId>,
        account_id: Option<LeaveAccountId>,
    ) -> impl Future<Output = Result<Vec<LeaveAdjustment>, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAdjustmentId,
    ) -> impl Future<Output = Result<Option<LeaveAdjustment>, CoreError>> + Send;

    fn insert(
        &mut self,
        adjustment: &LeaveAdjustment,
    ) -> impl Future<Output = Result<LeaveAdjustment, CoreError>> + Send;

    fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAdjustmentId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
