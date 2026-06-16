use crate::{OrganizationId, ServiceRateId, ServiceRateUnit};

#[derive(Debug, Clone)]
pub struct CreateServiceRateCommand {
    pub organization_id: OrganizationId,
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
}

#[derive(Debug, Clone)]
pub struct UpdateServiceRateCommand {
    pub id: ServiceRateId,
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
}
