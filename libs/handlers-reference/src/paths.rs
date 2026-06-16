use axum_extra::routing::TypedPath;
use mestier_core::{EmployeeId, EquipmentId, OrganizationId, ServiceRateId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/employees")]
pub struct EmployeesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/employees/{employee_id}")]
pub struct EmployeePath {
    pub employee_id: EmployeeId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/equipment")]
pub struct EquipmentCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/equipment/{equipment_id}")]
pub struct EquipmentPath {
    pub equipment_id: EquipmentId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/service-rates")]
pub struct ServiceRatesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/service-rates/{service_rate_id}")]
pub struct ServiceRatePath {
    pub service_rate_id: ServiceRateId,
}
