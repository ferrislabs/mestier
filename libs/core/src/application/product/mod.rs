use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, Product, ProductId,
    application::MestierUseCase,
    domain::product::{
        commands::{CreateProductCommand, UpdateProductCommand},
        service::ProductService,
    },
};

impl MestierUseCase {
    #[transactional(product)]
    pub async fn create_product(
        &self,
        command: CreateProductCommand,
    ) -> Result<Product, CoreError> {
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

    #[transactional(product)]
    pub async fn update_product(
        &self,
        command: UpdateProductCommand,
    ) -> Result<Product, CoreError> {
        let mut service = ProductService::new(product_repository);
        service.update_product(command).await
    }

    #[transactional(product)]
    pub async fn soft_delete_product(&self, id: ProductId) -> Result<(), CoreError> {
        let mut service = ProductService::new(product_repository);
        service.soft_delete_product(id).await
    }
}
