use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    Customer, CustomerContact, CustomerContext, Organization, VatStatus,
    domain::organization::billing_settings::{BillingSettings, IssuerBilling},
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
pub struct SnapshotAddress {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
}

impl SnapshotAddress {
    pub fn is_postal(&self) -> bool {
        let filled =
            |value: &Option<String>| value.as_deref().is_some_and(|text| !text.trim().is_empty());
        filled(&self.line1) && filled(&self.city)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct IssuerSnapshot {
    pub name: String,
    pub legal_form: Option<String>,
    pub registration_number: Option<String>,
    pub vat_number: Option<String>,
    pub vat_exemption_basis: Option<String>,
    pub share_capital_cents: Option<i64>,
    pub address: SnapshotAddress,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub insurance_mention: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub late_penalty: String,
    pub recovery_fee_cents: i32,
    pub invoice_footer: String,
}

impl IssuerSnapshot {
    pub fn of(organization: &Organization, settings: &BillingSettings) -> Self {
        let billing = IssuerBilling::from(settings);
        let (vat_number, vat_exemption_basis) = match &organization.vat_status {
            Some(VatStatus::Subject { vat_number }) => (Some(vat_number.clone()), None),
            Some(VatStatus::NotSubject { basis }) => (None, Some(basis.clone())),
            None => (None, None),
        };

        Self {
            name: organization
                .legal_name
                .clone()
                .unwrap_or_else(|| organization.name.clone()),
            legal_form: organization.legal_form.clone(),
            registration_number: organization.registration_number.clone(),
            vat_number,
            vat_exemption_basis,
            share_capital_cents: organization.share_capital_cents,
            address: SnapshotAddress {
                line1: organization.address_line1.clone(),
                line2: organization.address_line2.clone(),
                postal_code: organization.address_postal_code.clone(),
                city: organization.address_city.clone(),
                country: organization.address_country.clone(),
            },
            contact_email: organization.contact_email.clone(),
            contact_phone: organization.contact_phone.clone(),
            insurance_mention: organization.insurance_mention.clone(),
            iban: billing.iban,
            bic: billing.bic,
            late_penalty: billing.late_penalty,
            recovery_fee_cents: billing.recovery_fee_cents,
            invoice_footer: billing.invoice_footer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RecipientSnapshot {
    pub name: String,
    pub registration_number: Option<String>,
    pub vat_number: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub address: SnapshotAddress,
}

impl RecipientSnapshot {
    pub fn of(
        customer: &Customer,
        contact: Option<&CustomerContact>,
        address: Option<&CustomerContext>,
    ) -> Self {
        Self {
            name: customer.name.clone(),
            registration_number: customer.registration_number.clone(),
            vat_number: customer.profile.vat_number.clone(),
            contact_name: contact
                .map(|contact| format!("{} {}", contact.first_name, contact.last_name)),
            contact_email: contact.and_then(|contact| contact.email.clone()),
            address: address
                .map(|address| SnapshotAddress {
                    line1: address.address_line.clone(),
                    line2: address.address_line2.clone(),
                    postal_code: address.postal_code.clone(),
                    city: address.city.clone(),
                    country: Some(address.country.clone()),
                })
                .unwrap_or_default(),
        }
    }
}
