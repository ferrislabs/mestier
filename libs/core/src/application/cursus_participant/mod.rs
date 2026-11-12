use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CursusSpace, HostCompanyId, OrganizationId, Participant, ParticipantDetail, ParticipantId,
    ParticipantInvite, ParticipantListItem,
    application::{MestierUseCase, policy},
    domain::{
        cursus_host_company::service::CursusHostCompanyService,
        cursus_participant::{
            commands::{
                AcceptParticipantInvitationCommand, AddCompanyTutorCommand,
                CreateParticipantCommand, ParticipantFilter, RemoveCompanyTutorCommand,
                UpdateParticipantCommand, UpsertApprenticeCommand,
            },
            service::CursusParticipantService,
        },
        cursus_triad::service::CursusTriadService,
        member::ports::MemberRepository,
        organization::ports::OrganizationRepository,
    },
};

#[cfg(test)]
mod tests;

async fn require_participants_manager<M, R, A>(
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
        "cursus.participants.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

impl MestierUseCase {
    #[transactional(cursus_participant, cursus_host_company, role, member, authz, emitter)]
    pub async fn create_participant(
        &self,
        command: CreateParticipantCommand,
    ) -> Result<ParticipantDetail, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_participants_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        if let Some(tutor) = &command.company_tutor {
            ensure_host_company(
                cursus_host_company_repository,
                command.organization_id,
                tutor.host_company_id,
            )
            .await?;
        }

        CursusParticipantService::new(cursus_participant_repository, emitter)
            .create(command)
            .await
    }

    #[transactional(cursus_participant, emitter)]
    pub async fn get_participant(
        &self,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<ParticipantDetail, CoreError> {
        CursusParticipantService::new(cursus_participant_repository, emitter)
            .get_detail(organization_id, id)
            .await
    }

    #[transactional(cursus_participant, emitter)]
    pub async fn list_participants(
        &self,
        organization_id: OrganizationId,
        filter: ParticipantFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ParticipantListItem>, u64), CoreError> {
        CursusParticipantService::new(cursus_participant_repository, emitter)
            .list(organization_id, &filter, limit, offset)
            .await
    }

    #[transactional(cursus_participant, role, member, authz, emitter)]
    pub async fn update_participant(
        &self,
        command: UpdateParticipantCommand,
    ) -> Result<ParticipantDetail, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_participants_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusParticipantService::new(cursus_participant_repository, emitter)
            .update(command)
            .await
    }

    #[transactional(cursus_participant, role, member, authz, emitter)]
    pub async fn invite_participant(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<ParticipantInvite, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_participants_manager(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusParticipantService::new(cursus_participant_repository, emitter)
            .invite(organization_id, id)
            .await
    }

    #[transactional(cursus_participant, role, member, authz, emitter)]
    pub async fn deactivate_participant(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<Participant, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_participants_manager(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusParticipantService::new(cursus_participant_repository, emitter)
            .deactivate(organization_id, id)
            .await
    }

    #[transactional(cursus_participant, role, member, authz, emitter)]
    pub async fn reactivate_participant(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<Participant, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_participants_manager(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusParticipantService::new(cursus_participant_repository, emitter)
            .reactivate(organization_id, id)
            .await
    }

    #[transactional(cursus_participant, role, member, authz, emitter)]
    pub async fn upsert_apprentice(
        &self,
        command: UpsertApprenticeCommand,
    ) -> Result<ParticipantDetail, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_participants_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusParticipantService::new(cursus_participant_repository, emitter)
            .upsert_apprentice(command)
            .await
    }

    #[transactional(cursus_participant, cursus_host_company, role, member, authz, emitter)]
    pub async fn add_company_tutor(
        &self,
        command: AddCompanyTutorCommand,
    ) -> Result<ParticipantDetail, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_participants_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        ensure_host_company(
            cursus_host_company_repository,
            command.organization_id,
            command.company_tutor.host_company_id,
        )
        .await?;

        CursusParticipantService::new(cursus_participant_repository, emitter)
            .add_company_tutor(command)
            .await
    }

    #[transactional(cursus_participant, cursus_triad, role, member, authz, emitter)]
    pub async fn remove_company_tutor(
        &self,
        command: RemoveCompanyTutorCommand,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_participants_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        if CursusTriadService::new(cursus_triad_repository, emitter)
            .company_tutor_is_linked(command.organization_id, command.id)
            .await?
        {
            return Err(CoreError::Conflict(
                "company tutor is linked to a contract".to_owned(),
            ));
        }

        CursusParticipantService::new(cursus_participant_repository, emitter)
            .remove_company_tutor(command)
            .await
    }

    #[transactional(cursus_participant, organization, member, emitter)]
    pub async fn accept_participant_invitation(
        &self,
        command: AcceptParticipantInvitationCommand,
    ) -> Result<CursusSpace, CoreError> {
        let mut organization_repository = organization_repository;
        let mut member_repository = member_repository;
        let command_user_id = command.user_id;
        let participant = CursusParticipantService::new(cursus_participant_repository, emitter)
            .accept(command)
            .await?;

        let organization = organization_repository
            .find_by_id(participant.organization_id)
            .await?
            .ok_or(CoreError::Gone)?;

        let is_member = member_repository
            .find_by_org_and_user(organization.id, command_user_id)
            .await?
            .is_some();

        Ok(CursusSpace {
            organization_id: organization.id,
            slug: organization.slug,
            name: organization.name,
            is_member,
        })
    }
}

async fn ensure_host_company<R>(
    repository: R,
    organization_id: OrganizationId,
    id: HostCompanyId,
) -> Result<(), CoreError>
where
    R: crate::domain::cursus_host_company::ports::CursusHostCompanyRepository,
{
    CursusHostCompanyService::new(repository)
        .get(organization_id, id)
        .await
        .map(|_| ())
}
