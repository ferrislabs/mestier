use common::CoreError;
use mestier_macros::transactional;

use crate::{
    LeaveAdjustment, LeaveAdjustmentOutcome,
    application::{MestierUseCase, leave_account::require_permission, policy},
    domain::leave_adjustment::{
        LeaveAdjustmentService,
        commands::{
            CreateLeaveAdjustmentCommand, DeleteLeaveAdjustmentCommand, ListLeaveAdjustmentsCommand,
        },
    },
};

#[cfg(test)]
mod tests;

impl MestierUseCase {
    #[transactional(leave_adjustment, leave_account, member, role, authz)]
    pub async fn list_leave_adjustments(
        &self,
        command: ListLeaveAdjustmentsCommand,
    ) -> Result<Vec<LeaveAdjustment>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        match require_permission(
            command.actor.clone(),
            command.organization_id,
            "planning.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await
        {
            Ok(_) => {}
            Err(CoreError::Forbidden { .. }) => {
                require_permission(
                    command.actor,
                    command.organization_id,
                    "reference.manage",
                    &mut member_repository,
                    &mut role_repository,
                    &authz,
                )
                .await?;
            }
            Err(other) => return Err(other),
        }

        let mut service = LeaveAdjustmentService::new(
            leave_adjustment_repository,
            leave_account_repository,
            member_repository,
        );
        service
            .list(
                command.organization_id,
                command.member_id,
                command.account_id,
            )
            .await
    }

    #[transactional(leave_adjustment, leave_account, member, role, authz)]
    pub async fn create_leave_adjustment(
        &self,
        command: CreateLeaveAdjustmentCommand,
    ) -> Result<LeaveAdjustmentOutcome<LeaveAdjustment>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let actor = require_permission(
            command.actor,
            command.organization_id,
            "planning.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service = LeaveAdjustmentService::new(
            leave_adjustment_repository,
            leave_account_repository,
            member_repository,
        );
        service
            .create(
                command.organization_id,
                &command.draft,
                policy::subject_member_id(&actor),
            )
            .await
    }

    #[transactional(leave_adjustment, leave_account, member, role, authz)]
    pub async fn delete_leave_adjustment(
        &self,
        command: DeleteLeaveAdjustmentCommand,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_permission(
            command.actor,
            command.organization_id,
            "planning.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service = LeaveAdjustmentService::new(
            leave_adjustment_repository,
            leave_account_repository,
            member_repository,
        );
        service.delete(command.organization_id, command.id).await
    }
}
