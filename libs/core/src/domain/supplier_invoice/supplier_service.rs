use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    OrganizationId, Supplier, SupplierId,
    domain::supplier_invoice::{
        commands::{CreateSupplierCommand, UpdateSupplierCommand},
        ports::SupplierRepository,
    },
};

pub struct SupplierService<R>
where
    R: SupplierRepository,
{
    repo: R,
}

impl<R> SupplierService<R>
where
    R: SupplierRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_supplier(
        &mut self,
        command: CreateSupplierCommand,
    ) -> Result<Supplier, CoreError> {
        validate_name(&command.name)?;
        let email = normalize_optional(command.email);
        validate_email(email.as_deref())?;

        let now = Utc::now();
        self.repo
            .insert(&Supplier {
                id: SupplierId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: command.name.trim().to_owned(),
                registration_number: normalize_optional(command.registration_number),
                vat_number: normalize_optional(command.vat_number),
                address: normalize_optional(command.address),
                phone: normalize_optional(command.phone),
                email,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_supplier(&mut self, id: SupplierId) -> Result<Supplier, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_suppliers(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Supplier>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn update_supplier(
        &mut self,
        command: UpdateSupplierCommand,
    ) -> Result<Supplier, CoreError> {
        validate_name(&command.name)?;
        let email = normalize_optional(command.email);
        validate_email(email.as_deref())?;

        let mut supplier = self.get_supplier(command.id).await?;
        supplier.name = command.name.trim().to_owned();
        supplier.registration_number = normalize_optional(command.registration_number);
        supplier.vat_number = normalize_optional(command.vat_number);
        supplier.address = normalize_optional(command.address);
        supplier.phone = normalize_optional(command.phone);
        supplier.email = email;
        supplier.updated_at = Utc::now();

        self.repo.update(&supplier).await
    }
}

fn validate_name(name: &str) -> Result<(), CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::Conflict(
            "supplier name cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

fn validate_email(email: Option<&str>) -> Result<(), CoreError> {
    if email.is_some_and(|email| !email.contains('@')) {
        return Err(CoreError::Conflict(
            "supplier email must contain an @".to_owned(),
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
    use crate::MockSupplierRepository;

    fn create_command(name: &str, email: Option<&str>) -> CreateSupplierCommand {
        CreateSupplierCommand {
            actor: authz::Subject::system(),
            organization_id: OrganizationId(generate_uuid_v7()),
            name: name.to_owned(),
            registration_number: None,
            vat_number: None,
            address: Some(" 12 rue des Lilas ".to_owned()),
            phone: Some("  ".to_owned()),
            email: email.map(str::to_owned),
        }
    }

    #[tokio::test]
    async fn create_normalizes_contact_details() {
        let mut repo = MockSupplierRepository::new();
        repo.expect_insert().returning(|supplier| {
            let supplier = supplier.clone();
            Box::pin(async move { Ok(supplier) })
        });
        let mut service = SupplierService::new(repo);

        let supplier = service
            .create_supplier(create_command(" Point P ", Some("contact@pointp.fr")))
            .await
            .unwrap();

        assert_eq!(supplier.name, "Point P");
        assert_eq!(supplier.address.as_deref(), Some("12 rue des Lilas"));
        assert_eq!(supplier.phone, None);
        assert_eq!(supplier.email.as_deref(), Some("contact@pointp.fr"));
    }

    #[tokio::test]
    async fn create_refuses_a_blank_name() {
        let mut service = SupplierService::new(MockSupplierRepository::new());

        let result = service.create_supplier(create_command("  ", None)).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn create_refuses_an_email_without_at() {
        let mut service = SupplierService::new(MockSupplierRepository::new());

        let result = service
            .create_supplier(create_command("Point P", Some("pointp.fr")))
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }
}
