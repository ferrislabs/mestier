use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::{OrganizationId, ServiceRateId, ServiceRateUnit};

#[derive(Debug, Clone)]
pub struct CreateServiceRateCommand {
    pub organization_id: OrganizationId,
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
    pub vat_rate: Decimal,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct UpdateServiceRateCommand {
    pub id: ServiceRateId,
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
    pub vat_rate: Decimal,
    pub custom_fields: HashMap<String, String>,
}
