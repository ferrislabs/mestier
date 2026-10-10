use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CreateDoeCommand, Doe, DoeDocument, DoeId, DoeSummary, DoeSupply,
    DraftSupplyFromProductCommand, OrganizationId, ProjectId, UpdateDoeCommand,
    application::{MestierUseCase, policy},
    domain::doe::service::DoeService,
};

#[cfg(test)]
mod tests;

impl MestierUseCase {
    #[transactional(doe)]
    pub async fn get_project_doe(&self, project_id: ProjectId) -> Result<Doe, CoreError> {
        DoeService::new(doe_repository)
            .get_project_doe(project_id)
            .await
    }

    #[transactional(doe)]
    pub async fn list_does(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<DoeSummary>, u64), CoreError> {
        DoeService::new(doe_repository)
            .list(organization_id, limit, offset)
            .await
    }

    #[transactional(doe)]
    pub async fn get_doe(&self, id: DoeId) -> Result<Doe, CoreError> {
        DoeService::new(doe_repository).get_doe(id).await
    }

    #[transactional(doe, doe_model, doe_settings)]
    pub async fn doe_document(&self, id: DoeId) -> Result<DoeDocument, CoreError> {
        let mut doe_model_repository = doe_model_repository;
        let mut doe_settings_repository = doe_settings_repository;
        DoeService::new(doe_repository)
            .document(id, &mut doe_model_repository, &mut doe_settings_repository)
            .await
    }

    #[transactional(
        doe,
        doe_model,
        project,
        customer,
        customer_context,
        organization,
        member,
        role,
        authz
    )]
    pub async fn create_doe(&self, command: CreateDoeCommand) -> Result<Doe, CoreError> {
        let mut doe_model_repository = doe_model_repository;
        let mut project_repository = project_repository;
        let mut customer_repository = customer_repository;
        let mut customer_context_repository = customer_context_repository;
        let mut organization_repository = organization_repository;
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
            "planning.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        DoeService::new(doe_repository)
            .create_from_model(
                command,
                &mut doe_model_repository,
                &mut project_repository,
                &mut customer_repository,
                &mut customer_context_repository,
                &mut organization_repository,
            )
            .await
    }

    #[transactional(doe, member, role, authz)]
    pub async fn update_doe(&self, command: UpdateDoeCommand) -> Result<Doe, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut service = DoeService::new(doe_repository);

        let organization_id = service.get_doe(command.id).await?.organization_id;
        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await?;

        service.replace(command).await
    }

    #[transactional(doe, product, supplier, member, role, authz)]
    pub async fn draft_doe_supply(
        &self,
        command: DraftSupplyFromProductCommand,
    ) -> Result<DoeSupply, CoreError> {
        let mut product_repository = product_repository;
        let mut supplier_repository = supplier_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut service = DoeService::new(doe_repository);

        let organization_id = service.get_doe(command.doe_id).await?.organization_id;
        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await?;

        service
            .draft_supply_from_product(command, &mut product_repository, &mut supplier_repository)
            .await
    }
}
