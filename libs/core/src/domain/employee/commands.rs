use crate::{EmployeeId, OrganizationId, UserId};

#[derive(Debug, Clone)]
pub struct CreateEmployeeCommand {
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub name: String,
    /// `None` means the rate is not set yet; `Some(0)` means genuinely free.
    pub hourly_rate_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
}

#[derive(Debug, Clone)]
pub struct UpdateEmployeeCommand {
    pub id: EmployeeId,
    pub name: String,
    /// `None` means the rate is not set yet; `Some(0)` means genuinely free.
    pub hourly_rate_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
}

#[derive(Debug, Clone)]
pub struct LinkEmployeeUserCommand {
    pub id: EmployeeId,
    pub user_id: Option<UserId>,
}
