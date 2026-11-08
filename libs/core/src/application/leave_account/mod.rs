use authz::{Authorizer, Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    LeaveAccount, LeaveAccountOutcome, OrganizationId,
    application::{MestierUseCase, policy},
    domain::{
        leave_account::{
            LeaveAccountService,
            commands::{
                ArchiveLeaveAccountCommand, CreateLeaveAccountCommand, ListLeaveAccountsCommand,
                UpdateLeaveAccountCommand,
            },
        },
        member::ports::MemberRepository,
        role::ports::RoleRepository,
    },
};

#[cfg(test)]
mod tests;

pub(crate) async fn require_permission<M, R, A>(
    actor: Subject,
    organization_id: OrganizationId,
    action: &str,
    members: &mut M,
    roles: &mut R,
    authz: &A,
) -> Result<Subject, CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
    A: Authorizer,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authz,
        &actor,
        action,
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await?;
    Ok(actor)
}

impl MestierUseCase {
    #[transactional(leave_account, absence_type)]
    pub async fn list_leave_accounts(
        &self,
        command: ListLeaveAccountsCommand,
    ) -> Result<Vec<LeaveAccount>, CoreError> {
        let mut service =
            LeaveAccountService::new(leave_account_repository, absence_type_repository);
        service.list(command.organization_id).await
    }

    #[transactional(leave_account, absence_type, role, member, authz)]
    pub async fn create_leave_account(
        &self,
        command: CreateLeaveAccountCommand,
    ) -> Result<LeaveAccountOutcome<LeaveAccount>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_permission(
            command.actor,
            command.organization_id,
            "reference.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service =
            LeaveAccountService::new(leave_account_repository, absence_type_repository);
        service
            .create(command.organization_id, &command.draft)
            .await
    }

    #[transactional(leave_account, absence_type, role, member, authz)]
    pub async fn update_leave_account(
        &self,
        command: UpdateLeaveAccountCommand,
    ) -> Result<LeaveAccountOutcome<LeaveAccount>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_permission(
            command.actor,
            command.organization_id,
            "reference.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service =
            LeaveAccountService::new(leave_account_repository, absence_type_repository);
        service
            .update(command.organization_id, command.id, &command.draft)
            .await
    }

    #[transactional(leave_account, absence_type, role, member, authz)]
    pub async fn archive_leave_account(
        &self,
        command: ArchiveLeaveAccountCommand,
    ) -> Result<LeaveAccount, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_permission(
            command.actor,
            command.organization_id,
            "reference.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service =
            LeaveAccountService::new(leave_account_repository, absence_type_repository);
        service
            .set_archived(command.organization_id, command.id, command.archived)
            .await
    }
}
