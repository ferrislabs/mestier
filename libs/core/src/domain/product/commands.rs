use rust_decimal::Decimal;

use crate::{OrganizationId, ProductId, ServiceRateUnit};

#[derive(Debug, Clone)]
pub struct CreateProductCommand {
    pub organization_id: OrganizationId,
    pub name: String,
    pub sku: Option<String>,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub vat_rate: Decimal,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateProductCommand {
    pub id: ProductId,
    pub name: String,
    pub sku: Option<String>,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub vat_rate: Decimal,
    pub description: Option<String>,
}
