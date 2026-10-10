use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, Supplier, SupplierId,
    application::{MestierUseCase, policy},
    domain::supplier_invoice::{
        commands::{CreateSupplierCommand, UpdateSupplierCommand},
        supplier_service::SupplierService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(supplier, role, member, authz)]
    pub async fn create_supplier(
        &self,
        command: CreateSupplierCommand,
    ) -> Result<Supplier, CoreError> {
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

        let mut service = SupplierService::new(supplier_repository);
        service.create_supplier(command).await
    }

    #[transactional(supplier)]
    pub async fn get_supplier(&self, id: SupplierId) -> Result<Supplier, CoreError> {
        let mut service = SupplierService::new(supplier_repository);
        service.get_supplier(id).await
    }

    #[transactional(supplier)]
    pub async fn list_suppliers(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Supplier>, u64), CoreError> {
        let mut service = SupplierService::new(supplier_repository);
        service.list_suppliers(organization_id, limit, offset).await
    }

    #[transactional(supplier, role, member, authz)]
    pub async fn update_supplier(
        &self,
        command: UpdateSupplierCommand,
    ) -> Result<Supplier, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = SupplierService::new(supplier_repository);
        let existing = service.get_supplier(command.id).await?;

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

        service.update_supplier(command).await
    }
}
