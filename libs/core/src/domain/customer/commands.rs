use crate::{CustomerId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateCustomerCommand {
    pub organization_id: OrganizationId,
    pub last_name: String,
    pub first_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateCustomerCommand {
    pub id: CustomerId,
    pub last_name: String,
    pub first_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}
