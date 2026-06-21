use crate::{CustomerContactId, CustomerId};

#[derive(Debug, Clone)]
pub struct CreateCustomerContactCommand {
    pub customer_id: CustomerId,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateCustomerContactCommand {
    pub id: CustomerContactId,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}
