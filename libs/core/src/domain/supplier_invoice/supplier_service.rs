use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};

use super::profile::{
    SupplierError, SupplierField, SupplierOutcome, SupplierProfile, SupplierProfileInput,
    supplier_registration_number,
};
use crate::{
    OrganizationId, Supplier, SupplierId,
    domain::{
        expense::ports::ExpenseCategoryRepository,
        supplier_invoice::{
            commands::{CreateSupplierCommand, UpdateSupplierCommand},
            ports::SupplierRepository,
        },
    },
};

pub struct SupplierService<R, C>
where
    R: SupplierRepository,
    C: ExpenseCategoryRepository,
{
    repo: R,
    categories: C,
}

struct ValidSupplier {
    name: String,
    registration_number: Option<String>,
    vat_number: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    profile: SupplierProfile,
}

struct RawSupplier {
    name: String,
    registration_number: Option<String>,
    vat_number: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    profile: SupplierProfileInput,
}

fn validate(raw: RawSupplier) -> SupplierOutcome<ValidSupplier> {
    let name = raw.name.trim().to_owned();
    if name.is_empty() {
        return Err(SupplierError {
            field: SupplierField::Name,
            rule: "must not be empty",
        });
    }
    let email = normalize_optional(raw.email);
    if email.as_deref().is_some_and(|email| !email.contains('@')) {
        return Err(SupplierError {
            field: SupplierField::Email,
            rule: "must contain an @",
        });
    }
    Ok(ValidSupplier {
        name,
        registration_number: supplier_registration_number(raw.registration_number)?,
        vat_number: normalize_optional(raw.vat_number),
        address: normalize_optional(raw.address),
        phone: normalize_optional(raw.phone),
        email,
        profile: raw.profile.validated()?,
    })
}

impl<R, C> SupplierService<R, C>
where
    R: SupplierRepository,
    C: ExpenseCategoryRepository,
{
    pub fn new(repo: R, categories: C) -> Self {
        Self { repo, categories }
    }

    async fn require_category(
        &mut self,
        organization_id: OrganizationId,
        profile: &SupplierProfile,
    ) -> Result<(), CoreError> {
        let Some(category_id) = profile.default_category_id else {
            return Ok(());
        };
        self.categories
            .find_by_id(category_id)
            .await?
            .filter(|category| category.organization_id == organization_id)
            .map(|_| ())
            .ok_or(CoreError::NotFound)
    }

    pub async fn create_supplier(
        &mut self,
        command: CreateSupplierCommand,
    ) -> Result<SupplierOutcome<Supplier>, CoreError> {
        let valid = match validate(RawSupplier {
            name: command.name,
            registration_number: command.registration_number,
            vat_number: command.vat_number,
            address: command.address,
            phone: command.phone,
            email: command.email,
            profile: command.profile,
        }) {
            Ok(valid) => valid,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.require_category(command.organization_id, &valid.profile)
            .await?;

        let now = Utc::now();
        self.repo
            .insert(&Supplier {
                id: SupplierId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: valid.name,
                registration_number: valid.registration_number,
                vat_number: valid.vat_number,
                address: valid.address,
                phone: valid.phone,
                email: valid.email,
                profile: valid.profile,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
            .map(Ok)
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
    ) -> Result<SupplierOutcome<Supplier>, CoreError> {
        let valid = match validate(RawSupplier {
            name: command.name,
            registration_number: command.registration_number,
            vat_number: command.vat_number,
            address: command.address,
            phone: command.phone,
            email: command.email,
            profile: command.profile,
        }) {
            Ok(valid) => valid,
            Err(refusal) => return Ok(Err(refusal)),
        };

        let mut supplier = self.get_supplier(command.id).await?;
        self.require_category(supplier.organization_id, &valid.profile)
            .await?;
        supplier.name = valid.name;
        supplier.registration_number = valid.registration_number;
        supplier.vat_number = valid.vat_number;
        supplier.address = valid.address;
        supplier.phone = valid.phone;
        supplier.email = valid.email;
        supplier.profile = SupplierProfile {
            archived_at: supplier.profile.archived_at,
            ..valid.profile
        };
        supplier.updated_at = Utc::now();

        self.repo.update(&supplier).await.map(Ok)
    }

    pub async fn set_archived(
        &mut self,
        id: SupplierId,
        archived: bool,
        now: DateTime<Utc>,
    ) -> Result<Supplier, CoreError> {
        self.get_supplier(id).await?;
        self.repo
            .set_archived(id, archived.then_some(now), now)
            .await
    }
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
mod tests;
