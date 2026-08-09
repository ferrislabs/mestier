use common::CoreError;
use mestier_macros::transactional;

use crate::{
    UserId,
    application::MestierUseCase,
    domain::{
        member::{
            Member, MemberId,
            commands::{
                AddMemberCommand, AssignRoleCommand, CreateMemberCommand, UpdateMemberCommand,
            },
            service::MemberService,
        },
        organization::OrganizationId,
        role::RoleId,
    },
};

impl MestierUseCase {
    #[transactional(member)]
    pub async fn add_member(&self, command: AddMemberCommand) -> Result<Member, CoreError> {
        let mut service = MemberService::new(member_repository);
        service.add_member(command).await
    }

    /// A free, named seat — no `user_id`, unlike [`Self::add_member`].
    #[transactional(member)]
    pub async fn create_member(&self, command: CreateMemberCommand) -> Result<Member, CoreError> {
        let mut service = MemberService::new(member_repository);
        service.create_member(command).await
    }

    #[transactional(member)]
    pub async fn update_member(&self, command: UpdateMemberCommand) -> Result<Member, CoreError> {
        let mut service = MemberService::new(member_repository);
        service.update_member(command).await
    }

    /// `CoreError::NotFound` when no active seat exists for `member_id`.
    /// Mirrors `MestierUseCase::get_employee`.
    #[transactional(member)]
    pub async fn get_member(&self, member_id: MemberId) -> Result<Member, CoreError> {
        let mut service = MemberService::new(member_repository);
        service.get_member(member_id).await
    }

    #[transactional(member)]
    pub async fn list_members(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Member>, u64), CoreError> {
        let mut service = MemberService::new(member_repository);
        service.list_members(organization_id, limit, offset).await
    }

    #[transactional(member)]
    pub async fn find_membership(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Member>, CoreError> {
        let mut service = MemberService::new(member_repository);
        service.find_membership(organization_id, user_id).await
    }

    #[transactional(member)]
    pub async fn remove_member(&self, member_id: MemberId) -> Result<(), CoreError> {
        let mut service = MemberService::new(member_repository);
        service.remove_member(member_id).await
    }

    #[transactional(member)]
    pub async fn assign_role(&self, command: AssignRoleCommand) -> Result<(), CoreError> {
        let mut service = MemberService::new(member_repository);
        service.assign_role(command).await
    }

    #[transactional(member)]
    pub async fn list_role_ids(&self, member_id: MemberId) -> Result<Vec<RoleId>, CoreError> {
        let mut service = MemberService::new(member_repository);
        service.list_role_ids(member_id).await
    }
}
