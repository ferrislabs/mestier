use crate::UserId;
use chrono::{DateTime, Utc};
pub use common::OrganizationId;

pub mod commands;
pub mod legal_identity;
pub mod ports;
pub mod service;

pub use legal_identity::{
    ElectronicInvoicingFacts, LegalIdentity, MissingElectronicInvoicingFact,
    MissingLegalIdentityField, OrganizationAddress, VatStatus,
};

#[derive(Debug, Clone)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub owner_id: UserId,
    /// Legal identity, editable independently of `name`/`slug` (see
    /// `UpdateLegalIdentityCommand`). Every field below stays optional at
    /// this level — an organization mid-onboarding has none of them — and
    /// completeness is asserted only when a document is about to be issued,
    /// via `LegalIdentity::try_from_organization`.
    pub legal_name: Option<String>,
    pub legal_form: Option<String>,
    pub registration_number: Option<String>,
    pub vat_status: Option<VatStatus>,
    pub share_capital_cents: Option<i64>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub insurance_mention: Option<String>,
    /// The `{prefix}` in a quote number's `{prefix}-{year}-{counter}`
    /// shape. Always present — `DEV` by default, from the column default —
    /// unlike the legal-identity fields above: it blocks nothing, so
    /// giving it one is free.
    pub quote_number_prefix: String,
    /// The `{prefix}` in an invoice number's `{prefix}-{year}-{counter}`
    /// shape — `invoice_number_prefix`'s counterpart on the quote side.
    /// `FAC` by default, from the column default added alongside
    /// `invoice_number_counters` in #316; wired through here in #317, the
    /// first issue that actually allocates an invoice number.
    pub invoice_number_prefix: String,
    /// Whether the field app's home screen offers clocking in/out at all —
    /// `false` by default since ADR 0002: nothing computes money from
    /// `time_entries` any more, so a punch clock is opt-in, not a worker's
    /// first impression of the product. See the migration comment on
    /// `organizations.field_clock_enabled`.
    pub field_clock_enabled: bool,
    /// Whether this organization's VAT is due on invoicing ("debits")
    /// rather than on collection ("encaissements", the default under
    /// `false`). A property of the issuer's VAT regime, not a fact that can
    /// be missing — always `false` until set, never `Option`. The
    /// e-invoicing reform effective 1 September 2026 requires stating this
    /// explicitly on the invoice; see #341 and
    /// `legal_identity::ElectronicInvoicingFacts`.
    pub vat_on_debits: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn organization_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = OrganizationId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn organization_id_rejects_invalid_uuid() {
        assert!(OrganizationId::from_str("not-a-uuid").is_err());
    }
}
