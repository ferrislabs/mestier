use authz::Subject;

use crate::{CustomerContactId, CustomerId};

#[derive(Debug, Clone)]
pub struct CreateCustomerContactCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
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
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub id: CustomerContactId,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}
