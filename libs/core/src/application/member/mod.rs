use authz::Subject;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    UserId,
    application::MestierUseCase,
    domain::{
        member::{
            Member, MemberId, MemberWithAccount,
            commands::{
                AddMemberCommand, AssignRoleCommand, CreateMemberCommand, UpdateMemberCommand,
            },
            events::MemberRemoved,
            ports::MemberRepository,
            service::MemberService,
        },
        organization::OrganizationId,
        role::RoleId,
    },
};

impl MestierUseCase {
    #[transactional(member, role, user, authz)]
    pub async fn add_member(&self, command: AddMemberCommand) -> Result<Member, CoreError> {
        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);
        service.add_member(command).await
    }

    /// A free, named seat — no `user_id`, unlike [`Self::add_member`].
    #[transactional(member, role, user, authz)]
    pub async fn create_member(&self, command: CreateMemberCommand) -> Result<Member, CoreError> {
        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);
        service.create_member(command).await
    }

    /// Returns the updated seat together with its occupant, hydrated in the
    /// same call — see `MemberService::update_member`.
    #[transactional(member, role, user, authz)]
    pub async fn update_member(
        &self,
        command: UpdateMemberCommand,
    ) -> Result<MemberWithAccount, CoreError> {
        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);
        service.update_member(command).await
    }

    /// `CoreError::NotFound` when no active seat exists for `member_id`.
    /// Authorization happens inside `MemberService::get_member`, against
    /// the loaded seat's organization — never a caller-supplied one.
    #[transactional(member, role, user, authz)]
    pub async fn get_member(
        &self,
        actor: Subject,
        member_id: MemberId,
    ) -> Result<MemberWithAccount, CoreError> {
        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);
        service.get_member(actor, member_id).await
    }

    #[transactional(member, role, user, authz)]
    pub async fn list_members(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<MemberWithAccount>, u64), CoreError> {
        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);
        service
            .list_members(actor, organization_id, limit, offset)
            .await
    }

    /// The seat as stored, with no authorization of its own.
    ///
    /// For guards that are handed a bare `member_id` and immediately check its
    /// organization against the one the route already established — the check
    /// is the caller's, so doing a second one here would only hide which is
    /// load-bearing. Never expose this straight to a route.
    #[transactional(member)]
    pub async fn find_member_seat(&self, member_id: MemberId) -> Result<Option<Member>, CoreError> {
        let mut member_repository = member_repository;
        member_repository.find_by_id(member_id).await
    }

    #[transactional(member, role, user, authz)]
    pub async fn find_membership(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Member>, CoreError> {
        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);
        service.find_membership(organization_id, user_id).await
    }

    /// Authorization happens inside `MemberService::remove_member`, against
    /// the loaded seat's organization — never a caller-supplied one.
    #[transactional(member, role, user, authz, emitter)]
    pub async fn remove_member(
        &self,
        actor: Subject,
        member_id: MemberId,
    ) -> Result<(), CoreError> {
        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);
        let removed = service.remove_member(actor, member_id).await?;

        emitter.emit(removed.organization_id, &MemberRemoved { member: removed })?;

        Ok(())
    }

    #[transactional(member, role, user, authz)]
    pub async fn assign_role(&self, command: AssignRoleCommand) -> Result<(), CoreError> {
        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);
        service.assign_role(command).await
    }

    #[transactional(member, role, user, authz)]
    pub async fn list_role_ids(
        &self,
        member_id: MemberId,
        actor: Subject,
    ) -> Result<Vec<RoleId>, CoreError> {
        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);
        service.list_role_ids(member_id, actor).await
    }
}
