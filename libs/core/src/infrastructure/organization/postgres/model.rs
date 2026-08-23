use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    UserId,
    domain::organization::{Organization, OrganizationId, legal_identity::VatStatus},
};

#[derive(Debug, Clone)]
pub struct OrganizationRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub owner_id: Uuid,
    pub legal_name: Option<String>,
    pub legal_form: Option<String>,
    pub registration_number: Option<String>,
    pub vat_status: Option<String>,
    pub vat_number: Option<String>,
    pub vat_exemption_basis: Option<String>,
    pub share_capital_cents: Option<i64>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub insurance_mention: Option<String>,
    pub quote_number_prefix: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OrganizationRow {
    pub fn into_organization(self) -> Result<Organization, CoreError> {
        let vat_status =
            parse_vat_status(self.vat_status, self.vat_number, self.vat_exemption_basis)?;

        Ok(Organization {
            id: OrganizationId(self.id),
            name: self.name,
            slug: self.slug,
            owner_id: UserId(self.owner_id),
            legal_name: self.legal_name,
            legal_form: self.legal_form,
            registration_number: self.registration_number,
            vat_status,
            share_capital_cents: self.share_capital_cents,
            address_line1: self.address_line1,
            address_line2: self.address_line2,
            address_postal_code: self.address_postal_code,
            address_city: self.address_city,
            address_country: self.address_country,
            contact_email: self.contact_email,
            contact_phone: self.contact_phone,
            insurance_mention: self.insurance_mention,
            quote_number_prefix: self.quote_number_prefix,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// The database is the source of truth for the `subject`/`not_subject`
/// split (`chk_organizations_vat_number_when_subject` and its sibling
/// guarantee a row can never be saved half-filled), but this mapping is
/// still fallible: a row is data, not a type, and nothing stops another
/// process from writing to the table directly.
fn parse_vat_status(
    status: Option<String>,
    vat_number: Option<String>,
    basis: Option<String>,
) -> Result<Option<VatStatus>, CoreError> {
    match status.as_deref() {
        None => Ok(None),
        Some("subject") => vat_number
            .map(|vat_number| VatStatus::Subject { vat_number })
            .ok_or_else(|| {
                CoreError::Internal(
                    "organization vat_status is `subject` but vat_number is missing in database"
                        .to_owned(),
                )
            })
            .map(Some),
        Some("not_subject") => basis
            .map(|basis| VatStatus::NotSubject { basis })
            .ok_or_else(|| {
                CoreError::Internal(
                    "organization vat_status is `not_subject` but vat_exemption_basis is missing in database"
                        .to_owned(),
                )
            })
            .map(Some),
        Some(other) => Err(CoreError::Internal(format!(
            "invalid organization vat_status in database: {other}"
        ))),
    }
}

/// The reverse of `parse_vat_status`: the three columns to bind for a
/// `VatStatus`, or `NULL` all around when the organization has none yet.
pub fn vat_status_columns(
    vat_status: &Option<VatStatus>,
) -> (Option<&str>, Option<&str>, Option<&str>) {
    match vat_status {
        Some(VatStatus::Subject { vat_number }) => {
            (Some("subject"), Some(vat_number.as_str()), None)
        }
        Some(VatStatus::NotSubject { basis }) => (Some("not_subject"), None, Some(basis.as_str())),
        None => (None, None, None),
    }
}
