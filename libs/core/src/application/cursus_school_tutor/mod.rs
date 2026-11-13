use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    MemberId, OrganizationId, SchoolTutorId, SchoolTutorView,
    application::{MestierUseCase, policy},
    domain::{
        cursus_school_tutor::{
            commands::{CreateSchoolTutorCommand, SchoolTutorFilter, UpdateSchoolTutorCommand},
            service::CursusSchoolTutorService,
        },
        member::ports::MemberRepository,
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
    M: MemberRepository,
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

async fn ensure_member<M>(
    members: &mut M,
    organization_id: OrganizationId,
    id: MemberId,
) -> Result<(), CoreError>
where
    M: MemberRepository,
{
    members
        .find_by_id(id)
        .await?
        .filter(|member| member.organization_id == organization_id && member.deleted_at.is_none())
        .map(|_| ())
        .ok_or(CoreError::NotFound)
}

impl MestierUseCase {
    #[transactional(cursus_school_tutor, role, member, authz, emitter)]
    pub async fn create_school_tutor(
        &self,
        command: CreateSchoolTutorCommand,
    ) -> Result<SchoolTutorView, CoreError> {
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
        ensure_member(
            &mut member_repository,
            command.organization_id,
            command.member_id,
        )
        .await?;

        CursusSchoolTutorService::new(cursus_school_tutor_repository, emitter)
            .create(command)
            .await
    }

    #[transactional(cursus_school_tutor, emitter)]
    pub async fn get_school_tutor(
        &self,
        organization_id: OrganizationId,
        id: SchoolTutorId,
    ) -> Result<SchoolTutorView, CoreError> {
        CursusSchoolTutorService::new(cursus_school_tutor_repository, emitter)
            .get(organization_id, id)
            .await
    }

    #[transactional(cursus_school_tutor, emitter)]
    pub async fn list_school_tutors(
        &self,
        organization_id: OrganizationId,
        filter: SchoolTutorFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SchoolTutorView>, u64), CoreError> {
        CursusSchoolTutorService::new(cursus_school_tutor_repository, emitter)
            .list(organization_id, &filter, limit, offset)
            .await
    }

    #[transactional(cursus_school_tutor, role, member, authz, emitter)]
    pub async fn update_school_tutor(
        &self,
        command: UpdateSchoolTutorCommand,
    ) -> Result<SchoolTutorView, CoreError> {
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

        CursusSchoolTutorService::new(cursus_school_tutor_repository, emitter)
            .update(command)
            .await
    }

    #[transactional(cursus_school_tutor, role, member, authz, emitter)]
    pub async fn archive_school_tutor(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: SchoolTutorId,
    ) -> Result<SchoolTutorView, CoreError> {
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

        CursusSchoolTutorService::new(cursus_school_tutor_repository, emitter)
            .archive(organization_id, id)
            .await
    }

    #[transactional(cursus_school_tutor, role, member, authz, emitter)]
    pub async fn unarchive_school_tutor(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: SchoolTutorId,
    ) -> Result<SchoolTutorView, CoreError> {
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

        CursusSchoolTutorService::new(cursus_school_tutor_repository, emitter)
            .unarchive(organization_id, id)
            .await
    }
}
