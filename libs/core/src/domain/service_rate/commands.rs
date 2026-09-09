use crate::{OrganizationId, ServiceRateId};

#[derive(Debug, Clone)]
pub struct CreateServiceRateCommand {
    /// Authenticated actor performing the update. Built by the handler from
    /// the request `Identity`; carries the AuthZen-shaped subject the policy
    /// engine consumes.
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub label: String,
    pub unit: String,
    pub rate_cents: i32,
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateServiceRateCommand {
    /// Authenticated actor performing the update. Built by the handler from
    /// the request `Identity`; carries the AuthZen-shaped subject the policy
    /// engine consumes.
    pub actor: authz::Subject,
    pub id: ServiceRateId,
    pub label: String,
    pub unit: String,
    pub rate_cents: i32,
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
}
