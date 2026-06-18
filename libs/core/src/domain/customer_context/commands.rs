use crate::{CustomerContextId, CustomerId};

#[derive(Debug, Clone)]
pub struct CreateCustomerContextCommand {
    pub customer_id: CustomerId,
    pub label: String,
    pub address_line: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub photo_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateCustomerContextCommand {
    pub id: CustomerContextId,
    pub label: String,
    pub address_line: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub photo_key: Option<String>,
}
