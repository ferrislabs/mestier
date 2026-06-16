use crate::{EmployeeId, OrganizationId, UserId};

#[derive(Debug, Clone)]
pub struct CreateEmployeeCommand {
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub name: String,
    pub hourly_rate_cents: i32,
}

#[derive(Debug, Clone)]
pub struct UpdateEmployeeCommand {
    pub id: EmployeeId,
    pub name: String,
    pub hourly_rate_cents: i32,
}

#[derive(Debug, Clone)]
pub struct LinkEmployeeUserCommand {
    pub id: EmployeeId,
    pub user_id: Option<UserId>,
}
