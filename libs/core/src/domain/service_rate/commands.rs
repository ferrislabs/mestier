use rust_decimal::Decimal;

use crate::{OrganizationId, ServiceRateId, ServiceRateUnit};

#[derive(Debug, Clone)]
pub struct CreateServiceRateCommand {
    pub organization_id: OrganizationId,
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
    pub vat_rate: Decimal,
}

#[derive(Debug, Clone)]
pub struct UpdateServiceRateCommand {
    pub id: ServiceRateId,
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
    pub vat_rate: Decimal,
}
