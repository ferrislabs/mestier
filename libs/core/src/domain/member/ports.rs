use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    UserId,
    domain::{
        member::{Member, MemberId},
        organization::OrganizationId,
        role::RoleId,
    },
};

#[cfg_attr(test, mockall::automock)]
pub trait MemberRepository: Send {
    fn insert(&mut self, member: &Member)
    -> impl Future<Output = Result<Member, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        member_id: MemberId,
    ) -> impl Future<Output = Result<Option<Member>, CoreError>> + Send;

    /// Active members (`deleted_at IS NULL`) only.
    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Member>, CoreError>> + Send;

    /// Active members (`deleted_at IS NULL`) only.
    fn find_by_org_and_user(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Member>, CoreError>> + Send;

    fn update(&mut self, member: &Member)
    -> impl Future<Output = Result<Member, CoreError>> + Send;

    /// Signature mirrors `EmployeeRepository::soft_delete`. Replaces the
    /// former hard `remove`: a seat's history (roles today, HR data from
    /// #182 on) must survive someone leaving.
    fn soft_delete(
        &mut self,
        member_id: MemberId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn assign_role(
        &mut self,
        member_id: MemberId,
        role_id: RoleId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_role_ids(
        &mut self,
        member_id: MemberId,
    ) -> impl Future<Output = Result<Vec<RoleId>, CoreError>> + Send;
}
