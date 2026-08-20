use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Employee, EmployeeId, MemberId, OrganizationId};

#[derive(Debug, Clone)]
pub struct EmployeeRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub hourly_rate_cents: Option<i32>,
    pub is_salaried: bool,
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
            member_id: MemberId(row.member_id),
            hourly_rate_cents: row.hourly_rate_cents,
            is_salaried: row.is_salaried,
            weekly_contract_minutes: row.weekly_contract_minutes,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
