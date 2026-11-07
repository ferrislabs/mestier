use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    Customer, CustomerContactId, CustomerContextId, CustomerId, CustomerKind, CustomerProfile,
    CustomerStatus, LeadSource, MemberId, OrganizationId, PaymentTermsDays,
};

#[derive(Debug, Clone)]
pub struct CustomerRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub status: String,
    pub name: String,
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub kind: String,
    pub website: Option<String>,
    pub vat_number: Option<String>,
    pub owner_member_id: Option<Uuid>,
    pub lead_source: Option<String>,
    pub notes: Option<String>,
    pub payment_terms_days: Option<i32>,
    pub billing_contact_id: Option<Uuid>,
    pub billing_address_id: Option<Uuid>,
    pub converted_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<CustomerRow> for Customer {
    type Error = CoreError;

    fn try_from(row: CustomerRow) -> Result<Self, Self::Error> {
        let status = CustomerStatus::from_str(&row.status).map_err(|e| {
            CoreError::Internal(format!("invalid customer status in database: {e}"))
        })?;

        let kind = CustomerKind::from_str(&row.kind)
            .map_err(|e| CoreError::Internal(format!("invalid customer kind in database: {e}")))?;
        let lead_source = row
            .lead_source
            .as_deref()
            .map(LeadSource::from_str)
            .transpose()
            .map_err(|e| CoreError::Internal(format!("invalid lead source in database: {e}")))?;
        let payment_terms = row
            .payment_terms_days
            .map(|days| PaymentTermsDays::try_from(i64::from(days)))
            .transpose()
            .map_err(|e| CoreError::Internal(format!("invalid payment terms in database: {e}")))?;

        Ok(Self {
            id: CustomerId(row.id),
            organization_id: OrganizationId(row.org_id),
            status,
            name: row.name,
            registration_number: row.registration_number,
            phone: row.phone,
            email: row.email,
            profile: CustomerProfile {
                kind,
                website: row.website,
                vat_number: row.vat_number,
                owner_member_id: row.owner_member_id.map(MemberId),
                lead_source,
                notes: row.notes,
                payment_terms,
                billing_contact_id: row.billing_contact_id.map(CustomerContactId),
                billing_address_id: row.billing_address_id.map(CustomerContextId),
            },
            converted_at: row.converted_at,
            archived_at: row.archived_at,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
