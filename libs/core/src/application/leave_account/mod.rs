use common::CoreError;
use mestier_macros::transactional;

use crate::{
    LeaveAccount, LeaveAccountOutcome,
    application::{MestierUseCase, policy},
    domain::leave_account::{
        LeaveAccountService,
        commands::{
            ArchiveLeaveAccountCommand, CreateLeaveAccountCommand, ListLeaveAccountsCommand,
            UpdateLeaveAccountCommand,
        },
    },
};

#[cfg(test)]
mod tests;

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
        policy::require_permission(
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
        policy::require_permission(
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
        policy::require_permission(
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
