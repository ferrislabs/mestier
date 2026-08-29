use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Absence, AbsenceId, OrganizationId,
    application::{MestierUseCase, policy},
    domain::absence::{
        commands::{CreateAbsenceCommand, PatchAbsenceCommand},
        service::AbsenceService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(absence, role, member, authz)]
    pub async fn create_absence(
        &self,
        command: CreateAbsenceCommand,
    ) -> Result<Absence, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let mut service = AbsenceService::new(absence_repository);
        service.create_absence(command).await
    }

    #[transactional(absence)]
    pub async fn get_absence(&self, id: AbsenceId) -> Result<Absence, CoreError> {
        let mut service = AbsenceService::new(absence_repository);
        service.get_absence(id).await
    }

    #[transactional(absence)]
    pub async fn list_absences(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Absence>, u64), CoreError> {
        let mut service = AbsenceService::new(absence_repository);
        service.list_absences(organization_id, limit, offset).await
    }

    /// The absence row is loaded first and authorization runs against *its
    /// own* `organization_id`, never one taken from the request path — a
    /// bare `/absences/{id}` route has no organization to trust otherwise
    /// (CLAUDE.md's "bare ids derive their organization from the loaded
    /// row" rule).
    #[transactional(absence, role, member, authz)]
    pub async fn patch_absence(&self, command: PatchAbsenceCommand) -> Result<Absence, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = AbsenceService::new(absence_repository);
        let existing = service.get_absence(command.id).await?;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        service.patch_absence(command).await
    }

    /// Same "load, then authorize against the loaded row's own organization"
    /// rule as [`Self::patch_absence`] — there is no domain command to
    /// carry an `actor` for a bare-id delete, so it is threaded as its own
    /// parameter instead, the same way `remove_employee_profile` does.
    #[transactional(absence, role, member, authz)]
    pub async fn soft_delete_absence(
        &self,
        actor: Subject,
        id: AbsenceId,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = AbsenceService::new(absence_repository);
        let existing = service.get_absence(id).await?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        service.soft_delete_absence(id).await
    }
}
