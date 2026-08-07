use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Employee, EmployeeId, OrganizationId, UserId};

#[derive(Debug, Clone)]
pub struct EmployeeRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub hourly_rate_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EmployeeRow> for Employee {
    fn from(row: EmployeeRow) -> Self {
        Self {
            id: EmployeeId(row.id),
            organization_id: OrganizationId(row.org_id),
            user_id: row.user_id.map(UserId),
            name: row.name,
            hourly_rate_cents: row.hourly_rate_cents,
            weekly_contract_minutes: row.weekly_contract_minutes,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
