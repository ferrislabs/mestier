use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Closure, ContractFileKind, ContractId, ContractView, CursusSpace, FileKey, MemberId, MyTriad,
    MyTriadDetail, OrganizationId, ParticipantScope, Triad, TriadDetail, TriadId, TriadSummary,
    UserId,
    application::{MestierUseCase, policy},
    domain::{
        cursus_catalog::service::CursusCatalogService,
        cursus_host_company::service::CursusHostCompanyService,
        cursus_participant::service::CursusParticipantService,
        cursus_triad::{
            commands::{
                CloseContractCommand, CloseTriadCommand, CreateContractCommand, CreateTriadCommand,
                SetContractFileCommand, SetContractTutorsCommand, TriadFilter,
                UpdateContractCommand, UpdateTriadCommand,
            },
            service::CursusTriadService,
        },
        member::ports::MemberRepository,
        organization::ports::OrganizationRepository,
    },
};

#[cfg(test)]
mod tests;

async fn require_triads_manager<M, R, A>(
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
    authz: &A,
) -> Result<(), CoreError>
where
    M: MemberRepository,
    R: crate::domain::role::ports::RoleRepository,
    A: authz::Authorizer,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authz,
        &actor,
        "cursus.triads.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

async fn ensure_member<M>(
    members: &mut M,
    organization_id: OrganizationId,
    id: Option<MemberId>,
) -> Result<(), CoreError>
where
    M: MemberRepository,
{
    let Some(id) = id else {
        return Ok(());
    };

    members
        .find_by_id(id)
        .await?
        .filter(|member| member.organization_id == organization_id && member.deleted_at.is_none())
        .map(|_| ())
        .ok_or(CoreError::NotFound)
}

async fn ensure_open_host_company<R>(
    repository: R,
    organization_id: OrganizationId,
    id: crate::HostCompanyId,
) -> Result<(), CoreError>
where
    R: crate::domain::cursus_host_company::ports::CursusHostCompanyRepository,
{
    let company = CursusHostCompanyService::new(repository)
        .get(organization_id, id)
        .await?;
    if company.archived_at.is_some() {
        return Err(CoreError::Conflict("host company is archived".to_owned()));
    }

    Ok(())
}

async fn closure_for<R>(
    repository: R,
    organization_id: OrganizationId,
    closed_at: chrono::NaiveDate,
    reason_id: crate::ClosureReasonId,
    note: Option<String>,
) -> Result<Closure, CoreError>
where
    R: crate::domain::cursus_catalog::ports::CursusCatalogRepository,
{
    let reason = CursusCatalogService::new(repository)
        .get_closure_reason(organization_id, reason_id)
        .await?;

    Closure::new(organization_id, closed_at, &reason, note)
}

impl MestierUseCase {
    #[transactional(
        cursus_triad,
        cursus_catalog,
        cursus_participant,
        role,
        member,
        authz,
        emitter
    )]
    pub async fn create_triad(&self, command: CreateTriadCommand) -> Result<Triad, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_triads_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .get_promotion(command.organization_id, command.promotion_id)
            .await?;
        CursusParticipantService::new(cursus_participant_repository, emitter)
            .get_apprentice(command.organization_id, command.apprentice_id)
            .await?;
        ensure_member(
            &mut member_repository,
            command.organization_id,
            command.pedagogic_tutor_member_id,
        )
        .await?;

        CursusTriadService::new(cursus_triad_repository, emitter)
            .create_triad(command)
            .await
    }

    #[transactional(cursus_triad, emitter)]
    pub async fn get_triad(
        &self,
        organization_id: OrganizationId,
        id: TriadId,
    ) -> Result<TriadDetail, CoreError> {
        CursusTriadService::new(cursus_triad_repository, emitter)
            .get_triad_detail(organization_id, id)
            .await
    }

    #[transactional(cursus_triad, emitter)]
    pub async fn list_triads(
        &self,
        organization_id: OrganizationId,
        filter: TriadFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<TriadSummary>, u64), CoreError> {
        CursusTriadService::new(cursus_triad_repository, emitter)
            .list_triads(organization_id, &filter, limit, offset)
            .await
    }

    #[transactional(cursus_triad, role, member, authz, emitter)]
    pub async fn update_triad(
        &self,
        command: UpdateTriadCommand,
    ) -> Result<TriadDetail, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_triads_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;
        ensure_member(
            &mut member_repository,
            command.organization_id,
            command.pedagogic_tutor_member_id,
        )
        .await?;

        let organization_id = command.organization_id;
        let mut service = CursusTriadService::new(cursus_triad_repository, emitter);
        let updated = service.update_triad(command).await?;

        service.get_triad_detail(organization_id, updated.id).await
    }

    #[transactional(cursus_triad, cursus_catalog, role, member, authz, emitter)]
    pub async fn close_triad(&self, command: CloseTriadCommand) -> Result<TriadDetail, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_triads_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let closure = closure_for(
            cursus_catalog_repository,
            command.organization_id,
            command.closed_at,
            command.closure_reason_id,
            command.closure_note,
        )
        .await?;

        let mut service = CursusTriadService::new(cursus_triad_repository, emitter);
        service
            .close_triad(command.organization_id, command.id, closure)
            .await?;

        service
            .get_triad_detail(command.organization_id, command.id)
            .await
    }

    #[transactional(cursus_triad, cursus_host_company, role, member, authz, emitter)]
    pub async fn create_contract(
        &self,
        command: CreateContractCommand,
    ) -> Result<ContractView, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_triads_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;
        ensure_open_host_company(
            cursus_host_company_repository,
            command.organization_id,
            command.host_company_id,
        )
        .await?;

        let organization_id = command.organization_id;
        let mut service = CursusTriadService::new(cursus_triad_repository, emitter);
        let created = service.create_contract(command).await?;

        service.get_contract_view(organization_id, created.id).await
    }

    #[transactional(cursus_triad, emitter)]
    pub async fn list_contracts(
        &self,
        organization_id: OrganizationId,
        triad_id: TriadId,
    ) -> Result<Vec<ContractView>, CoreError> {
        CursusTriadService::new(cursus_triad_repository, emitter)
            .list_contract_views(organization_id, triad_id)
            .await
    }

    #[transactional(cursus_triad, emitter)]
    pub async fn get_contract(
        &self,
        organization_id: OrganizationId,
        id: ContractId,
    ) -> Result<ContractView, CoreError> {
        CursusTriadService::new(cursus_triad_repository, emitter)
            .get_contract_view(organization_id, id)
            .await
    }

    #[transactional(cursus_triad, cursus_host_company, role, member, authz, emitter)]
    pub async fn update_contract(
        &self,
        command: UpdateContractCommand,
    ) -> Result<ContractView, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_triads_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let organization_id = command.organization_id;
        let mut service = CursusTriadService::new(cursus_triad_repository, emitter);
        let current = service.get_contract(organization_id, command.id).await?;
        if current.host_company_id != command.host_company_id {
            ensure_open_host_company(
                cursus_host_company_repository,
                organization_id,
                command.host_company_id,
            )
            .await?;
        }

        let updated = service.update_contract(command).await?;

        service.get_contract_view(organization_id, updated.id).await
    }

    #[transactional(cursus_triad, cursus_catalog, role, member, authz, emitter)]
    pub async fn close_contract(
        &self,
        command: CloseContractCommand,
    ) -> Result<ContractView, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_triads_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let closure = closure_for(
            cursus_catalog_repository,
            command.organization_id,
            command.closed_at,
            command.closure_reason_id,
            command.closure_note,
        )
        .await?;

        let mut service = CursusTriadService::new(cursus_triad_repository, emitter);
        service
            .close_contract(command.organization_id, command.id, closure)
            .await?;

        service
            .get_contract_view(command.organization_id, command.id)
            .await
    }

    #[transactional(cursus_triad, cursus_participant, role, member, authz, emitter)]
    pub async fn set_contract_tutors(
        &self,
        command: SetContractTutorsCommand,
    ) -> Result<ContractView, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_triads_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut participants =
            CursusParticipantService::new(cursus_participant_repository, emitter);
        let mut tutors = Vec::with_capacity(command.tutors.len());
        for input in &command.tutors {
            let tutor = participants
                .get_company_tutor(command.organization_id, input.company_tutor_id)
                .await?;
            tutors.push((tutor, input.is_main));
        }

        let mut service = CursusTriadService::new(cursus_triad_repository, emitter);
        service
            .set_tutors(command.organization_id, command.contract_id, &tutors)
            .await?;

        service
            .get_contract_view(command.organization_id, command.contract_id)
            .await
    }

    #[transactional(cursus_triad, role, member, authz, emitter)]
    pub async fn set_contract_file(
        &self,
        command: SetContractFileCommand,
    ) -> Result<ContractView, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_triads_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let organization_id = command.organization_id;
        let contract_id = command.contract_id;
        let mut service = CursusTriadService::new(cursus_triad_repository, emitter);
        service.set_file(command).await?;

        service
            .get_contract_view(organization_id, contract_id)
            .await
    }

    #[transactional(cursus_triad, emitter)]
    pub async fn get_contract_file_key(
        &self,
        organization_id: OrganizationId,
        id: ContractId,
        kind: ContractFileKind,
    ) -> Result<FileKey, CoreError> {
        CursusTriadService::new(cursus_triad_repository, emitter)
            .get_file_key(organization_id, id, kind)
            .await
    }

    #[transactional(cursus_triad, cursus_participant, emitter)]
    pub async fn participant_scope(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<ParticipantScope, CoreError> {
        let participant = CursusParticipantService::new(cursus_participant_repository, emitter)
            .active_participant(organization_id, user_id)
            .await?;

        CursusTriadService::new(cursus_triad_repository, emitter)
            .participant_scope(organization_id, participant.id)
            .await
    }

    #[transactional(cursus_triad, cursus_participant, emitter)]
    pub async fn list_my_triads(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<MyTriad>, CoreError> {
        let participant = CursusParticipantService::new(cursus_participant_repository, emitter)
            .active_participant(organization_id, user_id)
            .await?;

        let mut service = CursusTriadService::new(cursus_triad_repository, emitter);
        let scope = service
            .participant_scope(organization_id, participant.id)
            .await?;

        service.my_triads(&scope).await
    }

    #[transactional(cursus_triad, cursus_participant, emitter)]
    pub async fn get_my_triad(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
        triad_id: TriadId,
    ) -> Result<MyTriadDetail, CoreError> {
        let participant = CursusParticipantService::new(cursus_participant_repository, emitter)
            .active_participant(organization_id, user_id)
            .await?;

        let mut service = CursusTriadService::new(cursus_triad_repository, emitter);
        let scope = service
            .participant_scope(organization_id, participant.id)
            .await?;

        service.my_triad(&scope, triad_id).await
    }

    #[transactional(cursus_participant, organization, member, emitter)]
    pub async fn list_my_cursus_spaces(
        &self,
        user_id: UserId,
    ) -> Result<Vec<CursusSpace>, CoreError> {
        let mut organization_repository = organization_repository;
        let mut member_repository = member_repository;
        let participations = CursusParticipantService::new(cursus_participant_repository, emitter)
            .list_active_participations(user_id)
            .await?;

        let mut spaces = Vec::with_capacity(participations.len());
        for participant in participations {
            let Some(organization) = organization_repository
                .find_by_id(participant.organization_id)
                .await?
            else {
                continue;
            };
            let is_member = member_repository
                .find_by_org_and_user(organization.id, user_id)
                .await?
                .is_some();
            spaces.push(CursusSpace {
                organization_id: organization.id,
                slug: organization.slug,
                name: organization.name,
                is_member,
            });
        }
        spaces.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(spaces)
    }
}
