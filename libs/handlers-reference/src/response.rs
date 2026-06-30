use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mestier_core::{
    BillingSettings, Employee, EmployeeId, Equipment, EquipmentId, LegalMentionTemplate,
    LegalMentionTemplateId, OrganizationId, Product, ProductId, ServiceRate, ServiceRateId,
    ServiceRateUnit, UserId,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct EmployeeResponse {
    pub id: EmployeeId,
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub name: String,
    pub hourly_rate_cents: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Employee> for EmployeeResponse {
    fn from(value: Employee) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            user_id: value.user_id,
            name: value.name,
            hourly_rate_cents: value.hourly_rate_cents,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct EquipmentResponse {
    pub id: EquipmentId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub hourly_rate_cents: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Equipment> for EquipmentResponse {
    fn from(value: Equipment) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            hourly_rate_cents: value.hourly_rate_cents,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ServiceRateResponse {
    pub id: ServiceRateId,
    pub organization_id: OrganizationId,
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
    pub vat_rate: String,
    pub custom_fields: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ServiceRate> for ServiceRateResponse {
    fn from(value: ServiceRate) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            label: value.label,
            unit: value.unit,
            rate_cents: value.rate_cents,
            vat_rate: value.vat_rate.normalize().to_string(),
            custom_fields: value.custom_fields,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProductResponse {
    pub id: ProductId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub sku: Option<String>,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub vat_rate: String,
    pub custom_fields: HashMap<String, String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Product> for ProductResponse {
    fn from(value: Product) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            sku: value.sku,
            unit: value.unit,
            unit_price_cents: value.unit_price_cents,
            vat_rate: value.vat_rate.normalize().to_string(),
            custom_fields: value.custom_fields,
            description: value.description,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// Decimal fields are serialized as normalized strings to avoid utoipa schema
/// issues (rust_decimal::Decimal does not implement ToSchema).
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct BillingSettingsResponse {
    pub org_id: OrganizationId,
    pub payment_terms_days: i32,
    pub late_penalty_rate: String,
    pub recovery_indemnity_cents: i32,
    pub default_deposit_basis: Option<String>,
    pub default_deposit_value: Option<String>,
    pub default_vat_rate: String,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub siret: Option<String>,
    pub rcs: Option<String>,
    pub ape: Option<String>,
    pub vat_intracom: Option<String>,
    pub footer: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BillingSettings> for BillingSettingsResponse {
    fn from(value: BillingSettings) -> Self {
        Self {
            org_id: value.org_id,
            payment_terms_days: value.payment_terms_days,
            late_penalty_rate: value.late_penalty_rate.normalize().to_string(),
            recovery_indemnity_cents: value.recovery_indemnity_cents,
            default_deposit_basis: value.default_deposit_basis,
            default_deposit_value: value
                .default_deposit_value
                .map(|d| d.normalize().to_string()),
            default_vat_rate: value.default_vat_rate.normalize().to_string(),
            iban: value.iban,
            bic: value.bic,
            siret: value.siret,
            rcs: value.rcs,
            ape: value.ape,
            vat_intracom: value.vat_intracom,
            footer: value.footer,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct LegalMentionTemplateResponse {
    pub id: LegalMentionTemplateId,
    pub org_id: OrganizationId,
    pub name: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<LegalMentionTemplate> for LegalMentionTemplateResponse {
    fn from(value: LegalMentionTemplate) -> Self {
        Self {
            id: value.id,
            org_id: value.org_id,
            name: value.name,
            body: value.body,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
