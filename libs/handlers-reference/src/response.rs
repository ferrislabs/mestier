use chrono::{DateTime, Utc};
use mestier_core::{
    Employee, EmployeeId, Equipment, EquipmentId, OrganizationId, ServiceRate, ServiceRateId,
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
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
