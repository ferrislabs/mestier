use crate::{OrganizationId, ProductId, ServiceRateUnit};

#[derive(Debug, Clone)]
pub struct CreateProductCommand {
    /// Authenticated actor performing the update. Built by the handler from
    /// the request `Identity`; carries the AuthZen-shaped subject the policy
    /// engine consumes.
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub sku: Option<String>,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateProductCommand {
    /// Authenticated actor performing the update. Built by the handler from
    /// the request `Identity`; carries the AuthZen-shaped subject the policy
    /// engine consumes.
    pub actor: authz::Subject,
    pub id: ProductId,
    pub name: String,
    pub sku: Option<String>,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
}
