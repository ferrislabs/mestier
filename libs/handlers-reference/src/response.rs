use chrono::{DateTime, Utc};
use mestier_core::{
    Employee, EmployeeId, Equipment, EquipmentId, MemberId, OrganizationId, Product, ProductId,
    ServiceRate, ServiceRateId, ServiceRateUnit, salaried_hourly_rate_cents,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
/// The contractual profile of a member. It carries no name and no account:
/// those belong to the seat, and duplicating them here is exactly what #182
/// removed.
pub struct EmployeeResponse {
    pub id: EmployeeId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    /// `null` means the rate is not set yet; `0` means genuinely free.
    pub hourly_rate_cents: Option<i32>,
    /// True when this person is costed from `monthly_cost_cents` rather than by
    /// the hour. Never means their time is free.
    pub is_salaried: bool,
    /// Monthly employer cost for a salaried person, `null` when not set.
    pub monthly_cost_cents: Option<i32>,
    /// What an hour of this person costs, whichever basis they are on: their own
    /// rate, or the equivalent derived from the monthly cost and the contract.
    ///
    /// Sent so the front never re-implements the division, and so a form can show
    /// the equivalent back to whoever typed the salary. `null` means the cost
    /// cannot be stated — an unset rate, or a salary with no contracted hours to
    /// spread it over.
    pub effective_hourly_rate_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Employee> for EmployeeResponse {
    fn from(value: Employee) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            member_id: value.member_id,
            hourly_rate_cents: value.hourly_rate_cents,
            is_salaried: value.is_salaried,
            monthly_cost_cents: value.monthly_cost_cents,
            effective_hourly_rate_cents: if value.is_salaried {
                salaried_hourly_rate_cents(value.monthly_cost_cents, value.weekly_contract_minutes)
            } else {
                value.hourly_rate_cents
            },
            weekly_contract_minutes: value.weekly_contract_minutes,
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
    pub default_vat_rate_bp: Option<i32>,
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
            default_vat_rate_bp: value.default_vat_rate_bp,
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
    pub default_vat_rate_bp: Option<i32>,
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
            default_vat_rate_bp: value.default_vat_rate_bp,
            description: value.description,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
