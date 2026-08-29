use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, Product, ProductId,
    application::{MestierUseCase, policy},
    domain::product::{
        commands::{CreateProductCommand, UpdateProductCommand},
        service::ProductService,
    },
};

impl MestierUseCase {
    #[transactional(product, role, member, authz)]
    pub async fn create_product(
        &self,
        command: CreateProductCommand,
    ) -> Result<Product, CoreError> {
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

        let mut service = ProductService::new(product_repository);
        service.create_product(command).await
    }

    #[transactional(product)]
    pub async fn get_product(&self, id: ProductId) -> Result<Product, CoreError> {
        let mut service = ProductService::new(product_repository);
        service.get_product(id).await
    }

    #[transactional(product)]
    pub async fn list_products(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Product>, u64), CoreError> {
        let mut service = ProductService::new(product_repository);
        service.list_products(organization_id, limit, offset).await
    }

    /// The product row is loaded first and authorization runs against *its
    /// own* `organization_id`, never one taken from the request path — a
    /// bare `/products/{id}` route has no organization to trust otherwise
    /// (CLAUDE.md's "bare ids derive their organization from the loaded
    /// row" rule).
    #[transactional(product, role, member, authz)]
    pub async fn update_product(
        &self,
        command: UpdateProductCommand,
    ) -> Result<Product, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = ProductService::new(product_repository);
        let existing = service.get_product(command.id).await?;

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

        service.update_product(command).await
    }

    /// Same "load, then authorize against the loaded row's own organization"
    /// rule as [`Self::update_product`] — there is no domain command to
    /// carry an `actor` for a bare-id delete, so it is threaded as its own
    /// parameter instead, the same way `remove_employee_profile` does.
    #[transactional(product, role, member, authz)]
    pub async fn soft_delete_product(
        &self,
        actor: Subject,
        id: ProductId,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = ProductService::new(product_repository);
        let existing = service.get_product(id).await?;

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

        service.soft_delete_product(id).await
    }
}
