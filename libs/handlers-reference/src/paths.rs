use axum_extra::routing::TypedPath;
use mestier_core::{EquipmentId, MemberId, OrganizationId, ProductId, ServiceRateId};
use serde::Deserialize;

/// A profile is a sub-resource of the seat it describes, never addressed on
/// its own — hence the member-scoped path and the absence of any
/// `/employees` collection.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/members/{member_id}/employee-profile")]
pub struct EmployeeProfilePath {
    pub member_id: MemberId,
}

/// The one bulk read over profiles, org-scoped rather than member-scoped —
/// callers who need every profile in an organization (the HR team table)
/// still can't address a profile on its own.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/employee-profiles")]
pub struct EmployeeProfilesCollectionPath {
    pub organization_id: OrganizationId,
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

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/products")]
pub struct ProductsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/products/{product_id}")]
pub struct ProductPath {
    pub product_id: ProductId,
}
