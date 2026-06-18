use chrono::{DateTime, Utc};
use mestier_core::{Customer, CustomerId, OrganizationId, Property, PropertyId};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CustomerResponse {
    pub id: CustomerId,
    pub organization_id: OrganizationId,
    pub last_name: String,
    pub first_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Customer> for CustomerResponse {
    fn from(value: Customer) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            last_name: value.last_name,
            first_name: value.first_name,
            phone: value.phone,
            email: value.email,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PropertyResponse {
    pub id: PropertyId,
    pub customer_id: CustomerId,
    pub label: String,
    pub street: String,
    pub zip: String,
    pub city: String,
    pub photo_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Property> for PropertyResponse {
    fn from(value: Property) -> Self {
        Self {
            id: value.id,
            customer_id: value.customer_id,
            label: value.label,
            street: value.street,
            zip: value.zip,
            city: value.city,
            photo_key: value.photo_key,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
