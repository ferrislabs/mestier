use axum_extra::routing::TypedPath;
use mestier_core::{OrganizationId, SupplierId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/suppliers")]
pub struct SuppliersPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/suppliers/{supplier_id}")]
pub struct SupplierPath {
    pub organization_id: OrganizationId,
    pub supplier_id: SupplierId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/archive")]
pub struct SupplierArchivePath {
    pub organization_id: OrganizationId,
    pub supplier_id: SupplierId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/restore")]
pub struct SupplierRestorePath {
    pub organization_id: OrganizationId,
    pub supplier_id: SupplierId,
}
