use chrono::NaiveDate;
use common::CoreError;

use super::{MemberContract, MemberContractId};
use crate::{MemberId, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MemberContractRepository: Send {
    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<MemberContract>, CoreError>> + Send;

    fn list_overlapping(
        &mut self,
        organization_id: OrganizationId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> impl Future<Output = Result<Vec<MemberContract>, CoreError>> + Send;

    fn list_by_member(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> impl Future<Output = Result<Vec<MemberContract>, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: MemberContractId,
    ) -> impl Future<Output = Result<Option<MemberContract>, CoreError>> + Send;

    fn insert(
        &mut self,
        contract: &MemberContract,
    ) -> impl Future<Output = Result<MemberContract, CoreError>> + Send;

    fn update(
        &mut self,
        contract: &MemberContract,
    ) -> impl Future<Output = Result<MemberContract, CoreError>> + Send;

    fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: MemberContractId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
