use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Surcharge, SurchargeOutcome,
    application::{MestierUseCase, leave_account::require_permission},
    domain::surcharge::{
        SurchargeService,
        commands::{
            ArchiveSurchargeCommand, CreateSurchargeCommand, ListSurchargesCommand,
            UpdateSurchargeCommand,
        },
    },
};

#[cfg(test)]
mod tests;

impl MestierUseCase {
    #[transactional(surcharge)]
    pub async fn list_surcharges(
        &self,
        command: ListSurchargesCommand,
    ) -> Result<Vec<Surcharge>, CoreError> {
        let mut service = SurchargeService::new(surcharge_repository);
        service.list(command.organization_id).await
    }

    #[transactional(surcharge, role, member, authz)]
    pub async fn create_surcharge(
        &self,
        command: CreateSurchargeCommand,
    ) -> Result<SurchargeOutcome<Surcharge>, CoreError> {
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

        let mut service = SurchargeService::new(surcharge_repository);
        service
            .create(command.organization_id, &command.draft)
            .await
    }

    #[transactional(surcharge, role, member, authz)]
    pub async fn update_surcharge(
        &self,
        command: UpdateSurchargeCommand,
    ) -> Result<SurchargeOutcome<Surcharge>, CoreError> {
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

        let mut service = SurchargeService::new(surcharge_repository);
        service
            .update(command.organization_id, command.id, &command.draft)
            .await
    }

    #[transactional(surcharge, role, member, authz)]
    pub async fn archive_surcharge(
        &self,
        command: ArchiveSurchargeCommand,
    ) -> Result<Surcharge, CoreError> {
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

        let mut service = SurchargeService::new(surcharge_repository);
        service
            .set_archived(command.organization_id, command.id, command.archived)
            .await
    }
}
