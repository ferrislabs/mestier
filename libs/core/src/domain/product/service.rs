use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    OrganizationId, Product, ProductId, SupplierId,
    domain::product::{
        commands::{CreateProductCommand, UpdateProductCommand},
        ports::ProductRepository,
    },
    domain::supplier_invoice::ports::SupplierRepository,
};

pub struct ProductService<R, S>
where
    R: ProductRepository,
    S: SupplierRepository,
{
    repo: R,
    suppliers: S,
}

impl<R, S> ProductService<R, S>
where
    R: ProductRepository,
    S: SupplierRepository,
{
    pub fn new(repo: R, suppliers: S) -> Self {
        Self { repo, suppliers }
    }

    async fn ensure_supplier(
        &mut self,
        organization_id: OrganizationId,
        supplier_id: Option<SupplierId>,
    ) -> Result<(), CoreError> {
        let Some(supplier_id) = supplier_id else {
            return Ok(());
        };
        match self.suppliers.find_by_id(supplier_id).await? {
            Some(supplier) if supplier.organization_id == organization_id => Ok(()),
            _ => Err(CoreError::NotFound),
        }
    }

    pub async fn create_product(
        &mut self,
        command: CreateProductCommand,
    ) -> Result<Product, CoreError> {
        validate_name(&command.name)?;
        validate_unit(&command.unit)?;
        validate_price(command.unit_price_cents)?;
        validate_vat_rate_bp(command.default_vat_rate_bp)?;
        validate_photo_keys(&command.photo_keys)?;
        validate_datasheet_keys(&command.datasheet_keys)?;
        self.ensure_supplier(command.organization_id, command.supplier_id)
            .await?;

        let now = Utc::now();
        self.repo
            .insert(&Product {
                id: ProductId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: command.name,
                sku: normalize_optional(command.sku),
                unit: command.unit,
                unit_price_cents: command.unit_price_cents,
                default_vat_rate_bp: command.default_vat_rate_bp,
                description: normalize_optional(command.description),
                photo_keys: command.photo_keys,
                supplier_id: command.supplier_id,
                manufacturer: normalize_optional(command.manufacturer),
                technical_specs: normalize_optional(command.technical_specs),
                datasheet_keys: command.datasheet_keys,
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
        validate_unit(&command.unit)?;
        validate_price(command.unit_price_cents)?;
        validate_vat_rate_bp(command.default_vat_rate_bp)?;
        validate_photo_keys(&command.photo_keys)?;
        validate_datasheet_keys(&command.datasheet_keys)?;

        let mut product = self.get_product(command.id).await?;
        self.ensure_supplier(product.organization_id, command.supplier_id)
            .await?;
        product.name = command.name;
        product.sku = normalize_optional(command.sku);
        product.unit = command.unit;
        product.unit_price_cents = command.unit_price_cents;
        product.default_vat_rate_bp = command.default_vat_rate_bp;
        product.description = normalize_optional(command.description);
        product.photo_keys = command.photo_keys;
        product.supplier_id = command.supplier_id;
        product.manufacturer = normalize_optional(command.manufacturer);
        product.technical_specs = normalize_optional(command.technical_specs);
        product.datasheet_keys = command.datasheet_keys;
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

fn validate_unit(unit: &str) -> Result<(), CoreError> {
    if unit.trim().is_empty() {
        return Err(CoreError::Conflict(
            "product unit cannot be empty".to_owned(),
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

fn validate_vat_rate_bp(default_vat_rate_bp: Option<i32>) -> Result<(), CoreError> {
    if default_vat_rate_bp.is_some_and(|rate_bp| !(0..=10_000).contains(&rate_bp)) {
        return Err(CoreError::Conflict(
            "default vat rate must be between 0 and 10000 basis points".to_owned(),
        ));
    }

    Ok(())
}

fn validate_photo_keys(photo_keys: &[String]) -> Result<(), CoreError> {
    if photo_keys.iter().any(|key| key.trim().is_empty()) {
        return Err(CoreError::Conflict(
            "product photo keys cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

fn validate_datasheet_keys(datasheet_keys: &[String]) -> Result<(), CoreError> {
    if datasheet_keys.iter().any(|key| key.trim().is_empty()) {
        return Err(CoreError::Conflict(
            "product datasheet keys cannot be empty".to_owned(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockProductRepository, MockSupplierRepository, Supplier};

    fn create_command(supplier_id: Option<SupplierId>) -> CreateProductCommand {
        CreateProductCommand {
            actor: authz::Subject::system(),
            organization_id: OrganizationId(generate_uuid_v7()),
            name: "Rosier".to_owned(),
            sku: None,
            unit: "u".to_owned(),
            unit_price_cents: 1200,
            default_vat_rate_bp: None,
            description: None,
            photo_keys: vec![],
            supplier_id,
            manufacturer: Some("  Pépinières Martin ".to_owned()),
            technical_specs: Some("   ".to_owned()),
            datasheet_keys: vec!["sheets/rosier.pdf".to_owned()],
        }
    }

    fn supplier(organization_id: OrganizationId) -> Supplier {
        let now = Utc::now();
        Supplier {
            id: SupplierId(generate_uuid_v7()),
            organization_id,
            name: "Point P".to_owned(),
            registration_number: None,
            vat_number: None,
            address: None,
            phone: None,
            email: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn echoing_products() -> MockProductRepository {
        let mut repo = MockProductRepository::new();
        repo.expect_insert().returning(|product| {
            let product = product.clone();
            Box::pin(async move { Ok(product) })
        });
        repo
    }

    #[tokio::test]
    async fn create_normalizes_the_technical_sheet() {
        let mut service = ProductService::new(echoing_products(), MockSupplierRepository::new());

        let product = service.create_product(create_command(None)).await.unwrap();

        assert_eq!(product.manufacturer.as_deref(), Some("Pépinières Martin"));
        assert_eq!(product.technical_specs, None);
        assert_eq!(product.datasheet_keys, vec!["sheets/rosier.pdf"]);
    }

    #[tokio::test]
    async fn create_accepts_a_supplier_of_the_same_organization() {
        let command = create_command(None);
        let found = supplier(command.organization_id);
        let command = CreateProductCommand {
            supplier_id: Some(found.id),
            ..command
        };
        let mut suppliers = MockSupplierRepository::new();
        suppliers.expect_find_by_id().returning(move |_| {
            let found = found.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        let mut service = ProductService::new(echoing_products(), suppliers);

        let product = service.create_product(command).await.unwrap();

        assert!(product.supplier_id.is_some());
    }

    #[tokio::test]
    async fn create_refuses_a_supplier_of_another_organization() {
        let other = supplier(OrganizationId(generate_uuid_v7()));
        let command = create_command(Some(other.id));
        let mut suppliers = MockSupplierRepository::new();
        suppliers.expect_find_by_id().returning(move |_| {
            let other = other.clone();
            Box::pin(async move { Ok(Some(other)) })
        });
        let mut service = ProductService::new(MockProductRepository::new(), suppliers);

        let result = service.create_product(command).await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn create_refuses_an_unknown_supplier() {
        let command = create_command(Some(SupplierId(generate_uuid_v7())));
        let mut suppliers = MockSupplierRepository::new();
        suppliers
            .expect_find_by_id()
            .returning(|_| Box::pin(async { Ok(None) }));
        let mut service = ProductService::new(MockProductRepository::new(), suppliers);

        let result = service.create_product(command).await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn create_refuses_a_blank_datasheet_key() {
        let command = CreateProductCommand {
            datasheet_keys: vec!["  ".to_owned()],
            ..create_command(None)
        };
        let mut service =
            ProductService::new(MockProductRepository::new(), MockSupplierRepository::new());

        let result = service.create_product(command).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }
}
