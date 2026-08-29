use authz::Subject;

use crate::{CustomerId, CustomerPipelineStage, CustomerStatus, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateCustomerCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub status: CustomerStatus,
    pub pipeline_stage: CustomerPipelineStage,
    pub name: String,
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateCustomerCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub id: CustomerId,
    pub status: CustomerStatus,
    pub pipeline_stage: CustomerPipelineStage,
    pub name: String,
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}
