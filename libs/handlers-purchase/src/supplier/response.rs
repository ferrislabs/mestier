use chrono::{DateTime, Utc};
use mestier_core::{OrganizationId, Supplier, SupplierId};
use serde::Serialize;
use utoipa::ToSchema;

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
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
