use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Product, ProductId,
    domain::product::{
        commands::{CreateProductCommand, UpdateProductCommand},
        ports::ProductRepository,
    },
};

pub struct ProductService<R>
where
    R: ProductRepository,
{
    repo: R,
}

impl<R> ProductService<R>
where
    R: ProductRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_product(
        &mut self,
        command: CreateProductCommand,
    ) -> Result<Product, CoreError> {
        validate_name(&command.name)?;
        validate_price(command.unit_price_cents)?;

        let now = Utc::now();
        self.repo
            .insert(&Product {
                id: ProductId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: command.name,
                sku: normalize_optional(command.sku),
                unit: command.unit,
                unit_price_cents: command.unit_price_cents,
                description: normalize_optional(command.description),
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_product(&mut self, id: ProductId) -> Result<Product, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_products(
        &mut self,
        organization_id: crate::OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Product>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn update_product(
        &mut self,
        command: UpdateProductCommand,
    ) -> Result<Product, CoreError> {
        validate_name(&command.name)?;
        validate_price(command.unit_price_cents)?;

        let mut product = self.get_product(command.id).await?;
        product.name = command.name;
        product.sku = normalize_optional(command.sku);
        product.unit = command.unit;
        product.unit_price_cents = command.unit_price_cents;
        product.description = normalize_optional(command.description);
        product.updated_at = Utc::now();

        self.repo.update(&product).await
    }

    pub async fn soft_delete_product(&mut self, id: ProductId) -> Result<(), CoreError> {
        self.get_product(id).await?;
        self.repo.soft_delete(id, Utc::now()).await
    }
}

fn validate_name(name: &str) -> Result<(), CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::Conflict(
            "product name cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

fn validate_price(unit_price_cents: i32) -> Result<(), CoreError> {
    if unit_price_cents < 0 {
        return Err(CoreError::Conflict(
            "product price cannot be negative".to_owned(),
        ));
    }

    Ok(())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}
