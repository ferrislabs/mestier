use chrono::{DateTime, Utc};
use mestier_core::{MissingLegalIdentityField, Organization, OrganizationId, UserId, VatStatus};
use serde::Serialize;
use utoipa::ToSchema;

/// Mirrors `mestier_core::VatStatus` at the wire boundary: a tagged enum so
/// a client can never receive a VAT number with no status, or a status with
/// no number/basis. See #310 for why this is not a nullable VAT number.
#[derive(Debug, Serialize, PartialEq, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VatStatusResponse {
    Subject { vat_number: String },
    NotSubject { basis: String },
}

impl From<VatStatus> for VatStatusResponse {
    fn from(value: VatStatus) -> Self {
        match value {
            VatStatus::Subject { vat_number } => Self::Subject { vat_number },
            VatStatus::NotSubject { basis } => Self::NotSubject { basis },
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct OrganizationResponse {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub owner_id: UserId,
    pub legal_name: Option<String>,
    pub legal_form: Option<String>,
    pub registration_number: Option<String>,
    pub vat_status: Option<VatStatusResponse>,
    pub share_capital_cents: Option<i64>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub insurance_mention: Option<String>,
    /// What still blocks this organization from issuing a legally valid
    /// document, in the same field-name vocabulary the PDF refusal (#314)
    /// uses — so the settings page (#311) can link straight to the field
    /// named in either place. Empty once the identity is complete.
    pub missing_legal_identity_fields: Vec<MissingLegalIdentityField>,
    /// Whether the field app's home screen offers clocking in/out — see
    /// `Organization::field_clock_enabled`'s own doc.
    pub field_clock_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Organization> for OrganizationResponse {
    fn from(org: Organization) -> Self {
        let missing_legal_identity_fields =
            mestier_core::LegalIdentity::try_from_organization(&org)
                .err()
                .unwrap_or_default();

        Self {
            id: org.id,
            name: org.name,
            slug: org.slug,
            owner_id: org.owner_id,
            legal_name: org.legal_name,
            legal_form: org.legal_form,
            registration_number: org.registration_number,
            vat_status: org.vat_status.map(Into::into),
            share_capital_cents: org.share_capital_cents,
            address_line1: org.address_line1,
            address_line2: org.address_line2,
            address_postal_code: org.address_postal_code,
            address_city: org.address_city,
            address_country: org.address_country,
            contact_email: org.contact_email,
            contact_phone: org.contact_phone,
            insurance_mention: org.insurance_mention,
            missing_legal_identity_fields,
            field_clock_enabled: org.field_clock_enabled,
            created_at: org.created_at,
            updated_at: org.updated_at,
        }
    }
}
