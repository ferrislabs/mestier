use axum_extra::routing::TypedPath;
use mestier_core::{CustomerContextId, CustomerId, OrganizationId};
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
#[typed_path("/api/v1/customers/{customer_id}/customer-contexts")]
pub struct CustomerContextsPath {
    pub customer_id: CustomerId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/customer-contexts/{customer_context_id}")]
pub struct CustomerContextPath {
    pub customer_context_id: CustomerContextId,
}
