use crate::{CustomerId, CustomerStatus, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateCustomerCommand {
    pub organization_id: OrganizationId,
    pub status: CustomerStatus,
    pub last_name: String,
    pub first_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateCustomerCommand {
    pub id: CustomerId,
    pub status: CustomerStatus,
    pub last_name: String,
    pub first_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}
