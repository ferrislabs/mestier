use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    ArchiveMaintenanceContractCommand, CreateMaintenanceContractCommand,
    DuplicateMaintenanceContractCommand, MaintenanceContract, MaintenanceContractId,
    OrganizationId, UpdateMaintenanceContractCommand,
    application::{MestierAuthorizer, MestierUseCase, policy},
    domain::{
        maintenance::contract_service::MaintenanceContractService, member::ports::MemberRepository,
        role::ports::RoleRepository,
    },
};

/// Every maintenance contract use case, reads included, needs
/// `planning.manage`: the contract carries the amount and the conducteur's
/// whole book of work.
async fn require_planning_manage<M, R>(
    authorizer: &MestierAuthorizer,
    actor: authz::Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
) -> Result<(), CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authorizer,
        &actor,
        "planning.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

impl MestierUseCase {
    #[transactional(
        maintenance_contract,
        customer,
        customer_context,
        project,
        member,
        role,
        quote,
        authz
    )]
    pub async fn create_maintenance_contract(
        &self,
        command: CreateMaintenanceContractCommand,
    ) -> Result<MaintenanceContract, CoreError> {
        let mut customer_repository = customer_repository;
        let mut customer_context_repository = customer_context_repository;
        let mut project_repository = project_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut quote_repository = quote_repository;

        require_planning_manage(
            authz,
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        MaintenanceContractService::new(maintenance_contract_repository)
            .create(
                command,
                &mut customer_repository,
                &mut customer_context_repository,
                &mut member_repository,
                &mut quote_repository,
                &mut project_repository,
            )
            .await
    }

    /// `NotFound` for a contract of another organization than
    /// `organization_id`, the one the request was authorized against.
    #[transactional(maintenance_contract, member, role, authz)]
    pub async fn get_maintenance_contract(
        &self,
        actor: authz::Subject,
        organization_id: OrganizationId,
        id: MaintenanceContractId,
    ) -> Result<MaintenanceContract, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        require_planning_manage(
            authz,
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        MaintenanceContractService::new(maintenance_contract_repository)
            .get(organization_id, id)
            .await
    }

    #[transactional(maintenance_contract, member, role, authz)]
    pub async fn list_maintenance_contracts(
        &self,
        actor: authz::Subject,
        organization_id: OrganizationId,
        year: i32,
        include_archived: bool,
    ) -> Result<Vec<MaintenanceContract>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        require_planning_manage(
            authz,
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        MaintenanceContractService::new(maintenance_contract_repository)
            .list(organization_id, year, include_archived)
            .await
    }

    #[transactional(maintenance_contract, member, role, quote, authz)]
    pub async fn update_maintenance_contract(
        &self,
        command: UpdateMaintenanceContractCommand,
    ) -> Result<MaintenanceContract, CoreError> {
        let mut maintenance_contract_repository = maintenance_contract_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut quote_repository = quote_repository;

        let organization_id =
            organization_of(&mut maintenance_contract_repository, command.id).await?;
        require_planning_manage(
            authz,
            command.actor.clone(),
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        MaintenanceContractService::new(maintenance_contract_repository)
            .update(command, &mut member_repository, &mut quote_repository)
            .await
    }

    #[transactional(maintenance_contract, member, role, authz)]
    pub async fn archive_maintenance_contract(
        &self,
        command: ArchiveMaintenanceContractCommand,
    ) -> Result<MaintenanceContract, CoreError> {
        let mut maintenance_contract_repository = maintenance_contract_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let organization_id =
            organization_of(&mut maintenance_contract_repository, command.id).await?;
        require_planning_manage(
            authz,
            command.actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        MaintenanceContractService::new(maintenance_contract_repository)
            .archive(command.id)
            .await
    }

    #[transactional(
        maintenance_contract,
        customer,
        customer_context,
        project,
        member,
        role,
        authz
    )]
    pub async fn duplicate_maintenance_contract(
        &self,
        command: DuplicateMaintenanceContractCommand,
    ) -> Result<MaintenanceContract, CoreError> {
        let mut maintenance_contract_repository = maintenance_contract_repository;
        let mut customer_repository = customer_repository;
        let mut customer_context_repository = customer_context_repository;
        let mut project_repository = project_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let organization_id =
            organization_of(&mut maintenance_contract_repository, command.id).await?;
        require_planning_manage(
            authz,
            command.actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        MaintenanceContractService::new(maintenance_contract_repository)
            .duplicate(
                command.id,
                &mut customer_repository,
                &mut customer_context_repository,
                &mut project_repository,
            )
            .await
    }
}

async fn organization_of<R>(
    contracts: &mut R,
    id: MaintenanceContractId,
) -> Result<OrganizationId, CoreError>
where
    R: crate::domain::maintenance::ports::MaintenanceContractRepository,
{
    contracts
        .find_by_id(id)
        .await?
        .map(|contract| contract.organization_id)
        .ok_or(CoreError::NotFound)
}
