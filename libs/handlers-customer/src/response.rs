use chrono::{DateTime, Utc};
use mestier_core::{
    Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId, CustomerId,
    CustomerPipelineStage, CustomerStatus, OrganizationId,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CustomerResponse {
    pub id: CustomerId,
    pub organization_id: OrganizationId,
    pub status: CustomerStatus,
    pub pipeline_stage: CustomerPipelineStage,
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
            status: value.status,
            pipeline_stage: value.pipeline_stage,
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
pub struct CustomerContactResponse {
    pub id: CustomerContactId,
    pub customer_id: CustomerId,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CustomerContact> for CustomerContactResponse {
    fn from(value: CustomerContact) -> Self {
        Self {
            id: value.id,
            customer_id: value.customer_id,
            first_name: value.first_name,
            last_name: value.last_name,
            role: value.role,
            phone: value.phone,
            email: value.email,
            is_primary: value.is_primary,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CustomerContextResponse {
    pub id: CustomerContextId,
    pub customer_id: CustomerId,
    pub label: String,
    pub address_line: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub photo_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CustomerContext> for CustomerContextResponse {
    fn from(value: CustomerContext) -> Self {
        Self {
            id: value.id,
            customer_id: value.customer_id,
            label: value.label,
            address_line: value.address_line,
            postal_code: value.postal_code,
            city: value.city,
            photo_key: value.photo_key,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
