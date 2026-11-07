use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    AddressKind, Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId,
    CustomerId, CustomerKind, CustomerListRow, CustomerStatus, Deal, DealId, DealStageName,
    LeadSource, MemberId, OrganizationId, PipelineSummary, PrimaryContact,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CustomerResponse {
    pub id: CustomerId,
    pub organization_id: OrganizationId,
    pub status: CustomerStatus,
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
pub struct PrimaryContactResponse {
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
}

impl From<PrimaryContact> for PrimaryContactResponse {
    fn from(value: PrimaryContact) -> Self {
        Self {
            first_name: value.first_name,
            last_name: value.last_name,
            role: value.role,
            email: value.email,
            phone: value.phone,
            mobile: value.mobile,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CustomerListItemResponse {
    #[serde(flatten)]
    pub customer: CustomerResponse,
    pub city: Option<String>,
    pub primary_contact: Option<PrimaryContactResponse>,
    pub revenue_12m_cents: i64,
    pub overdue_cents: i64,
    pub last_activity_at: Option<DateTime<Utc>>,
}

impl From<CustomerListRow> for CustomerListItemResponse {
    fn from(value: CustomerListRow) -> Self {
        Self {
            customer: CustomerResponse::from(value.customer),
            city: value.city,
            primary_contact: value.primary_contact.map(PrimaryContactResponse::from),
            revenue_12m_cents: value.revenue_12m_cents,
            overdue_cents: value.overdue_cents,
            last_activity_at: value.last_activity_at,
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

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DealResponse {
    pub id: DealId,
    pub organization_id: OrganizationId,
    pub customer_id: CustomerId,
    pub title: String,
    pub amount_cents: i64,
    pub stage: DealStageName,
    pub expected_close_date: Option<NaiveDate>,
    pub owner_member_id: Option<MemberId>,
    pub source: Option<LeadSource>,
    pub lost_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub stage_changed_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub quote_count: i64,
}

impl From<Deal> for DealResponse {
    fn from(value: Deal) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            customer_id: value.customer_id,
            title: value.title.into_inner(),
            amount_cents: value.amount.get(),
            stage: value.stage.name(),
            expected_close_date: value.expected_close_date,
            owner_member_id: value.owner_member_id,
            source: value.source,
            lost_reason: value
                .stage
                .lost_reason()
                .map(|reason| reason.as_str().to_owned()),
            notes: value.notes,
            created_at: value.created_at,
            stage_changed_at: value.stage_changed_at,
            closed_at: value.closed_at,
            quote_count: value.quote_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct StageTotalResponse {
    pub stage: DealStageName,
    pub count: i64,
    pub amount_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DealSummaryResponse {
    pub open_count: i64,
    pub open_amount_cents: i64,
    pub stages: Vec<StageTotalResponse>,
    pub won_count: i64,
    pub lost_count: i64,
    pub won_amount_cents: i64,
    pub win_rate: Option<u8>,
    pub conversions: i64,
}

impl From<PipelineSummary> for DealSummaryResponse {
    fn from(value: PipelineSummary) -> Self {
        Self {
            open_count: value.open_count,
            open_amount_cents: value.open_amount_cents,
            stages: value
                .stages
                .into_iter()
                .map(|total| StageTotalResponse {
                    stage: total.stage,
                    count: total.count,
                    amount_cents: total.amount_cents,
                })
                .collect(),
            won_count: value.won_count,
            lost_count: value.lost_count,
            won_amount_cents: value.won_amount_cents,
            win_rate: value.win_rate,
            conversions: value.conversions,
        }
    }
}
