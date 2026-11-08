use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Absence, AbsenceError, AbsenceId, AbsenceOutcome, OrganizationId,
    application::{MestierUseCase, organization::resolve_timezone, policy},
    domain::absence::{
        commands::{
            CreateAbsenceCommand, DecideAbsenceCommand, ListAbsencesCommand, PatchAbsenceCommand,
            RejectAbsenceCommand,
        },
        service::AbsenceService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(
        absence,
        absence_type,
        leave_account,
        organization,
        role,
        member,
        authz
    )]
    pub async fn create_absence(
        &self,
        command: CreateAbsenceCommand,
    ) -> Result<AbsenceOutcome<Absence>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut organization_repository = organization_repository;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        let creator = policy::subject_member_id(&actor);
        let privileged = policy::holds(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;
        if !privileged {
            if creator != Some(command.member_id) {
                return Err(CoreError::Forbidden {
                    reason: Some("a member can only request an absence for themselves".to_owned()),
                });
            }
            if command.type_id.is_none() {
                return Ok(Err(AbsenceError::TypeRequired));
            }
        }

        let timezone =
            resolve_timezone(&mut organization_repository, command.organization_id).await?;
        let mut service = AbsenceService::new(
            absence_repository,
            absence_type_repository,
            leave_account_repository,
        );
        service.create_absence(command, creator, timezone).await
    }

    #[transactional(absence, absence_type, leave_account, role, member, authz)]
    pub async fn list_absences_for(
        &self,
        command: ListAbsencesCommand,
    ) -> Result<(Vec<Absence>, u64), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        let resource = Resource::new("organization", command.organization_id.0.to_string());
        let organization_wide = policy::holds(&authz, &actor, "planning.manage", resource.clone())
            .await?
            || policy::holds(&authz, &actor, "reference.manage", resource).await?;
        let scope = policy::own_scope(&actor, command.member_id, organization_wide)?;

        let mut service = AbsenceService::new(
            absence_repository,
            absence_type_repository,
            leave_account_repository,
        );
        match scope {
            Some(member_id) => {
                service
                    .list_absences_of_member(
                        command.organization_id,
                        member_id,
                        command.limit,
                        command.offset,
                    )
                    .await
            }
            None => {
                service
                    .list_absences(command.organization_id, command.limit, command.offset)
                    .await
            }
        }
    }

    #[transactional(absence, absence_type, leave_account)]
    pub async fn get_absence(&self, id: AbsenceId) -> Result<Absence, CoreError> {
        let mut service = AbsenceService::new(
            absence_repository,
            absence_type_repository,
            leave_account_repository,
        );
        service.get_absence(id).await
    }

    #[transactional(absence, absence_type, leave_account)]
    pub async fn list_absences(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Absence>, u64), CoreError> {
        let mut service = AbsenceService::new(
            absence_repository,
            absence_type_repository,
            leave_account_repository,
        );
        service.list_absences(organization_id, limit, offset).await
    }

    /// The absence row is loaded first and authorization runs against *its
    /// own* `organization_id`, never one taken from the request path — a
    /// bare `/absences/{id}` route has no organization to trust otherwise
    /// (CLAUDE.md's "bare ids derive their organization from the loaded
    /// row" rule).
    #[transactional(
        absence,
        absence_type,
        leave_account,
        organization,
        role,
        member,
        authz
    )]
    pub async fn patch_absence(&self, command: PatchAbsenceCommand) -> Result<Absence, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut organization_repository = organization_repository;

        let mut service = AbsenceService::new(
            absence_repository,
            absence_type_repository,
            leave_account_repository,
        );
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

        let timezone =
            resolve_timezone(&mut organization_repository, existing.organization_id).await?;
        service.patch_absence(command, timezone).await
    }

    /// Same "load, then authorize against the loaded row's own organization"
    /// rule as [`Self::patch_absence`] — there is no domain command to
    /// carry an `actor` for a bare-id delete, so it is threaded as its own
    /// parameter instead, the same way `remove_employee_profile` does.
    #[transactional(absence, absence_type, leave_account, role, member, authz)]
    pub async fn soft_delete_absence(
        &self,
        actor: Subject,
        id: AbsenceId,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = AbsenceService::new(
            absence_repository,
            absence_type_repository,
            leave_account_repository,
        );
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

    #[transactional(absence, absence_type, leave_account, role, member, authz)]
    pub async fn approve_absence(
        &self,
        command: DecideAbsenceCommand,
    ) -> Result<AbsenceOutcome<Absence>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let mut service = AbsenceService::new(
            absence_repository,
            absence_type_repository,
            leave_account_repository,
        );
        service
            .approve(
                command.organization_id,
                command.id,
                policy::subject_member_id(&actor),
            )
            .await
    }

    #[transactional(absence, absence_type, leave_account, role, member, authz)]
    pub async fn reject_absence(
        &self,
        command: RejectAbsenceCommand,
    ) -> Result<AbsenceOutcome<Absence>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let mut service = AbsenceService::new(
            absence_repository,
            absence_type_repository,
            leave_account_repository,
        );
        service
            .reject(
                command.organization_id,
                command.id,
                &command.comment,
                policy::subject_member_id(&actor),
            )
            .await
    }

    /// The creator may withdraw their own request; anyone else needs
    /// `planning.manage`.
    #[transactional(absence, absence_type, leave_account, role, member, authz)]
    pub async fn cancel_absence(
        &self,
        command: DecideAbsenceCommand,
    ) -> Result<AbsenceOutcome<Absence>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        let privileged = match policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await
        {
            Ok(()) => true,
            Err(CoreError::Forbidden { .. }) => false,
            Err(other) => return Err(other),
        };

        let mut service = AbsenceService::new(
            absence_repository,
            absence_type_repository,
            leave_account_repository,
        );
        service
            .cancel(
                command.organization_id,
                command.id,
                policy::subject_member_id(&actor),
                privileged,
            )
            .await
    }
}
