use std::fmt;

use chrono::{DateTime, Utc};
use common::generate_uuid_v7;
use uuid::Uuid;

use crate::{AddressKind, ExpenseCategoryId};

pub const MAX_PAYMENT_TERMS_DAYS: i32 = 999;
const SIREN_LEN: usize = 9;
const SIRET_LEN: usize = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupplierField {
    Name,
    Email,
    RegistrationNumber,
    PaymentTermsDays,
    Contacts,
}

impl SupplierField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Email => "email",
            Self::RegistrationNumber => "registration_number",
            Self::PaymentTermsDays => "payment_terms_days",
            Self::Contacts => "contacts",
        }
    }
}

impl fmt::Display for SupplierField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{field}: {rule}")]
pub struct SupplierError {
    pub field: SupplierField,
    pub rule: &'static str,
}

pub type SupplierOutcome<T> = Result<T, SupplierError>;

#[derive(Debug, Clone, PartialEq)]
pub struct SupplierContact {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub email: String,
    pub phone: String,
    pub mobile: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SupplierAddress {
    pub id: Uuid,
    pub kind: AddressKind,
    pub label: String,
    pub street: String,
    pub line2: String,
    pub postal_code: String,
    pub city: String,
    pub country: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SupplierProfile {
    pub iban: Option<String>,
    pub payment_terms_days: Option<i32>,
    pub default_category_id: Option<ExpenseCategoryId>,
    pub notes: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub contacts: Vec<SupplierContact>,
    pub addresses: Vec<SupplierAddress>,
}

#[derive(Debug, Clone, Default)]
pub struct SupplierContactInput {
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub email: String,
    pub phone: String,
    pub mobile: String,
}

#[derive(Debug, Clone)]
pub struct SupplierAddressInput {
    pub kind: AddressKind,
    pub label: String,
    pub street: String,
    pub line2: String,
    pub postal_code: String,
    pub city: String,
    pub country: String,
}

#[derive(Debug, Clone, Default)]
pub struct SupplierProfileInput {
    pub iban: Option<String>,
    pub payment_terms_days: Option<i32>,
    pub default_category_id: Option<ExpenseCategoryId>,
    pub notes: Option<String>,
    pub contacts: Vec<SupplierContactInput>,
    pub addresses: Vec<SupplierAddressInput>,
}

impl From<&SupplierProfile> for SupplierProfileInput {
    fn from(profile: &SupplierProfile) -> Self {
        Self {
            iban: profile.iban.clone(),
            payment_terms_days: profile.payment_terms_days,
            default_category_id: profile.default_category_id,
            notes: profile.notes.clone(),
            contacts: profile
                .contacts
                .iter()
                .map(|contact| SupplierContactInput {
                    first_name: contact.first_name.clone(),
                    last_name: contact.last_name.clone(),
                    role: contact.role.clone(),
                    email: contact.email.clone(),
                    phone: contact.phone.clone(),
                    mobile: contact.mobile.clone(),
                })
                .collect(),
            addresses: profile
                .addresses
                .iter()
                .map(|address| SupplierAddressInput {
                    kind: address.kind,
                    label: address.label.clone(),
                    street: address.street.clone(),
                    line2: address.line2.clone(),
                    postal_code: address.postal_code.clone(),
                    city: address.city.clone(),
                    country: address.country.clone(),
                })
                .collect(),
        }
    }
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

pub fn supplier_registration_number(value: Option<String>) -> SupplierOutcome<Option<String>> {
    let Some(value) = clean(value) else {
        return Ok(None);
    };
    let digits: String = value.chars().filter(|c| !c.is_whitespace()).collect();
    let well_formed =
        digits.chars().all(|c| c.is_ascii_digit()) && matches!(digits.len(), SIREN_LEN | SIRET_LEN);
    if well_formed {
        Ok(Some(digits))
    } else {
        Err(SupplierError {
            field: SupplierField::RegistrationNumber,
            rule: "must hold 9 or 14 digits",
        })
    }
}

pub fn is_plausible_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !email.chars().any(char::is_whitespace)
        && !domain.contains('@')
        && domain
            .split_once('.')
            .is_some_and(|(host, tld)| !host.is_empty() && !tld.is_empty())
}

impl SupplierProfileInput {
    pub fn validated(self) -> SupplierOutcome<SupplierProfile> {
        if let Some(days) = self.payment_terms_days
            && !(0..=MAX_PAYMENT_TERMS_DAYS).contains(&days)
        {
            return Err(SupplierError {
                field: SupplierField::PaymentTermsDays,
                rule: "must be between 0 and 999 days",
            });
        }
        let mut contacts = Vec::with_capacity(self.contacts.len());
        for contact in self.contacts {
            let email = contact.email.trim().to_owned();
            if !email.is_empty() && !is_plausible_email(&email) {
                return Err(SupplierError {
                    field: SupplierField::Contacts,
                    rule: "a contact e-mail address is not valid",
                });
            }
            contacts.push(SupplierContact {
                id: generate_uuid_v7(),
                first_name: contact.first_name.trim().to_owned(),
                last_name: contact.last_name.trim().to_owned(),
                role: contact.role.trim().to_owned(),
                email,
                phone: contact.phone.trim().to_owned(),
                mobile: contact.mobile.trim().to_owned(),
            });
        }
        let addresses = self
            .addresses
            .into_iter()
            .map(|address| SupplierAddress {
                id: generate_uuid_v7(),
                kind: address.kind,
                label: address.label.trim().to_owned(),
                street: address.street.trim().to_owned(),
                line2: address.line2.trim().to_owned(),
                postal_code: address.postal_code.trim().to_owned(),
                city: address.city.trim().to_owned(),
                country: address.country.trim().to_owned(),
            })
            .collect();
        Ok(SupplierProfile {
            iban: clean(self.iban),
            payment_terms_days: self.payment_terms_days,
            default_category_id: self.default_category_id,
            notes: clean(self.notes),
            archived_at: None,
            contacts,
            addresses,
        })
    }
}
