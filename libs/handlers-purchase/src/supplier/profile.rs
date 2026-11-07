use mestier_core::{
    AddressKind, ExpenseCategoryId, SupplierAddress, SupplierAddressInput, SupplierContact,
    SupplierContactInput, SupplierProfileInput,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct SupplierContactRequest {
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub mobile: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SupplierAddressRequest {
    pub kind: AddressKind,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub street: String,
    #[serde(default)]
    pub line2: String,
    #[serde(default)]
    pub postal_code: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub country: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct SupplierContactResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub email: String,
    pub phone: String,
    pub mobile: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct SupplierAddressResponse {
    pub id: Uuid,
    pub kind: AddressKind,
    pub label: String,
    pub street: String,
    pub line2: String,
    pub postal_code: String,
    pub city: String,
    pub country: String,
}

impl From<SupplierContact> for SupplierContactResponse {
    fn from(value: SupplierContact) -> Self {
        Self {
            id: value.id,
            first_name: value.first_name,
            last_name: value.last_name,
            role: value.role,
            email: value.email,
            phone: value.phone,
            mobile: value.mobile,
        }
    }
}

impl From<SupplierAddress> for SupplierAddressResponse {
    fn from(value: SupplierAddress) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            label: value.label,
            street: value.street,
            line2: value.line2,
            postal_code: value.postal_code,
            city: value.city,
            country: value.country,
        }
    }
}

pub fn profile_input(
    iban: Option<String>,
    payment_terms_days: Option<i32>,
    default_category_id: Option<ExpenseCategoryId>,
    notes: Option<String>,
    contacts: Vec<SupplierContactRequest>,
    addresses: Vec<SupplierAddressRequest>,
) -> SupplierProfileInput {
    SupplierProfileInput {
        iban,
        payment_terms_days,
        default_category_id,
        notes,
        contacts: contacts
            .into_iter()
            .map(|contact| SupplierContactInput {
                first_name: contact.first_name,
                last_name: contact.last_name,
                role: contact.role,
                email: contact.email,
                phone: contact.phone,
                mobile: contact.mobile,
            })
            .collect(),
        addresses: addresses
            .into_iter()
            .map(|address| SupplierAddressInput {
                kind: address.kind,
                label: address.label,
                street: address.street,
                line2: address.line2,
                postal_code: address.postal_code,
                city: address.city,
                country: address.country,
            })
            .collect(),
    }
}
