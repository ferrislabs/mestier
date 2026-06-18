use crate::{CustomerId, PropertyId};

#[derive(Debug, Clone)]
pub struct CreatePropertyCommand {
    pub customer_id: CustomerId,
    pub label: String,
    pub street: String,
    pub zip: String,
    pub city: String,
    pub photo_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdatePropertyCommand {
    pub id: PropertyId,
    pub label: String,
    pub street: String,
    pub zip: String,
    pub city: String,
    pub photo_key: Option<String>,
}
