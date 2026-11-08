use common::CoreError;

use super::{LeaveAccount, LeaveAccountId};
use crate::OrganizationId;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait LeaveAccountRepository: Send {
    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<LeaveAccount>, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAccountId,
    ) -> impl Future<Output = Result<Option<LeaveAccount>, CoreError>> + Send;

    fn insert(
        &mut self,
        account: &LeaveAccount,
    ) -> impl Future<Output = Result<LeaveAccount, CoreError>> + Send;

    fn update(
        &mut self,
        account: &LeaveAccount,
    ) -> impl Future<Output = Result<LeaveAccount, CoreError>> + Send;
}
