use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    AbsenceType, AbsenceTypeOutcome, OrganizationId,
    application::{MestierUseCase, policy},
    domain::absence_type::{
        AbsenceTypeService,
        commands::{
            ArchiveAbsenceTypeCommand, CreateAbsenceTypeCommand, ListAbsenceTypesCommand,
            UpdateAbsenceTypeCommand,
        },
    },
};

#[cfg(test)]
mod tests;

async fn require_reference_manager<M, R, A>(
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
    authz: &A,
) -> Result<(), CoreError>
where
    M: crate::domain::member::ports::MemberRepository,
    R: crate::domain::role::ports::RoleRepository,
    A: authz::Authorizer,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authz,
        &actor,
        "reference.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

impl MestierUseCase {
    #[transactional(absence_type, leave_account)]
    pub async fn list_absence_types(
        &self,
        command: ListAbsenceTypesCommand,
    ) -> Result<Vec<AbsenceType>, CoreError> {
        let mut service =
            AbsenceTypeService::new(absence_type_repository, leave_account_repository);
        service.list(command.organization_id).await
    }

    #[transactional(absence_type, leave_account, role, member, authz)]
    pub async fn create_absence_type(
        &self,
        command: CreateAbsenceTypeCommand,
    ) -> Result<AbsenceTypeOutcome<AbsenceType>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_reference_manager(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service =
            AbsenceTypeService::new(absence_type_repository, leave_account_repository);
        service
            .create(command.organization_id, &command.draft)
            .await
    }

    #[transactional(absence_type, leave_account, role, member, authz)]
    pub async fn update_absence_type(
        &self,
        command: UpdateAbsenceTypeCommand,
    ) -> Result<AbsenceTypeOutcome<AbsenceType>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_reference_manager(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service =
            AbsenceTypeService::new(absence_type_repository, leave_account_repository);
        service
            .update(command.organization_id, command.id, &command.draft)
            .await
    }

    #[transactional(absence_type, leave_account, role, member, authz)]
    pub async fn archive_absence_type(
        &self,
        command: ArchiveAbsenceTypeCommand,
    ) -> Result<AbsenceType, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_reference_manager(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service =
            AbsenceTypeService::new(absence_type_repository, leave_account_repository);
        service
            .set_archived(command.organization_id, command.id, command.archived)
            .await
    }
}
