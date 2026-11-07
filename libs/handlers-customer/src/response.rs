use chrono::{DateTime, Utc};
use mestier_core::{
    AddressKind, Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId,
    CustomerId, CustomerKind, CustomerPipelineStage, CustomerStatus, LeadSource, MemberId,
    OrganizationId,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CustomerResponse {
    pub id: CustomerId,
    pub organization_id: OrganizationId,
    pub status: CustomerStatus,
    pub pipeline_stage: CustomerPipelineStage,
    pub name: String,
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub kind: CustomerKind,
    pub website: Option<String>,
    pub vat_number: Option<String>,
    pub owner_member_id: Option<MemberId>,
    pub lead_source: Option<LeadSource>,
    pub notes: Option<String>,
    pub payment_terms_days: Option<i32>,
    pub billing_contact_id: Option<CustomerContactId>,
    pub billing_address_id: Option<CustomerContextId>,
    pub converted_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
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
            name: value.name,
            registration_number: value.registration_number,
            phone: value.phone,
            email: value.email,
            kind: value.profile.kind,
            website: value.profile.website,
            vat_number: value.profile.vat_number,
            owner_member_id: value.profile.owner_member_id,
            lead_source: value.profile.lead_source,
            notes: value.profile.notes,
            payment_terms_days: value.profile.payment_terms.map(|days| days.get()),
            billing_contact_id: value.profile.billing_contact_id,
            billing_address_id: value.profile.billing_address_id,
            converted_at: value.converted_at,
            archived_at: value.archived_at,
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
    pub mobile: Option<String>,
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
            mobile: value.mobile,
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
    pub kind: AddressKind,
    pub address_line: Option<String>,
    pub address_line2: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country: String,
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
            kind: value.kind,
            address_line: value.address_line,
            address_line2: value.address_line2,
            postal_code: value.postal_code,
            city: value.city,
            country: value.country,
            photo_key: value.photo_key,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
