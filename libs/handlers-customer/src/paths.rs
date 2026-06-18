use axum_extra::routing::TypedPath;
use mestier_core::{CustomerId, OrganizationId, PropertyId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/customers")]
pub struct CustomersPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/customers/{customer_id}")]
pub struct CustomerPath {
    pub customer_id: CustomerId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/customers/{customer_id}/properties")]
pub struct PropertiesPath {
    pub customer_id: CustomerId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/properties/{property_id}")]
pub struct PropertyPath {
    pub property_id: PropertyId,
}
