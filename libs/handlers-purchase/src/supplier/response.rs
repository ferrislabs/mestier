use chrono::{DateTime, Utc};
use mestier_core::{ExpenseCategoryId, OrganizationId, Supplier, SupplierId};
use serde::Serialize;
use utoipa::ToSchema;

use super::profile::{SupplierAddressResponse, SupplierContactResponse};

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct SupplierResponse {
    pub id: SupplierId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub registration_number: Option<String>,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub iban: Option<String>,
    pub payment_terms_days: Option<i32>,
    pub default_category_id: Option<ExpenseCategoryId>,
    pub notes: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub contacts: Vec<SupplierContactResponse>,
    pub addresses: Vec<SupplierAddressResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Supplier> for SupplierResponse {
    fn from(value: Supplier) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            registration_number: value.registration_number,
            vat_number: value.vat_number,
            address: value.address,
            phone: value.phone,
            email: value.email,
            iban: value.profile.iban,
            payment_terms_days: value.profile.payment_terms_days,
            default_category_id: value.profile.default_category_id,
            notes: value.profile.notes,
            archived_at: value.profile.archived_at,
            contacts: value.profile.contacts.into_iter().map(Into::into).collect(),
            addresses: value
                .profile
                .addresses
                .into_iter()
                .map(Into::into)
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
