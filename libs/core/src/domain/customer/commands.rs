use crate::{CustomerId, CustomerPipelineStage, CustomerStatus, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateCustomerCommand {
    pub organization_id: OrganizationId,
    pub status: CustomerStatus,
    pub pipeline_stage: CustomerPipelineStage,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateCustomerCommand {
    pub id: CustomerId,
    pub status: CustomerStatus,
    pub pipeline_stage: CustomerPipelineStage,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}
