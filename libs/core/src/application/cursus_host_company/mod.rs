use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    HostCompany, HostCompanyId, OrganizationId,
    application::{MestierUseCase, policy},
    domain::cursus_host_company::{
        commands::{CreateHostCompanyCommand, UpdateHostCompanyCommand},
        service::CursusHostCompanyService,
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
    M: crate::domain::member::ports::MemberRepository,
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

impl MestierUseCase {
    #[transactional(cursus_host_company, role, member, authz)]
    pub async fn create_host_company(
        &self,
        command: CreateHostCompanyCommand,
    ) -> Result<HostCompany, CoreError> {
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

        CursusHostCompanyService::new(cursus_host_company_repository)
            .create(command)
            .await
    }

    #[transactional(cursus_host_company)]
    pub async fn get_host_company(
        &self,
        organization_id: OrganizationId,
        id: HostCompanyId,
    ) -> Result<HostCompany, CoreError> {
        CursusHostCompanyService::new(cursus_host_company_repository)
            .get(organization_id, id)
            .await
    }

    #[transactional(cursus_host_company)]
    pub async fn list_host_companies(
        &self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<HostCompany>, u64), CoreError> {
        CursusHostCompanyService::new(cursus_host_company_repository)
            .list(organization_id, include_archived, limit, offset)
            .await
    }

    #[transactional(cursus_host_company, role, member, authz)]
    pub async fn update_host_company(
        &self,
        command: UpdateHostCompanyCommand,
    ) -> Result<HostCompany, CoreError> {
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

        CursusHostCompanyService::new(cursus_host_company_repository)
            .update(command)
            .await
    }

    #[transactional(cursus_host_company, role, member, authz)]
    pub async fn archive_host_company(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: HostCompanyId,
    ) -> Result<HostCompany, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_triads_manager(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusHostCompanyService::new(cursus_host_company_repository)
            .archive(organization_id, id)
            .await
    }
}
