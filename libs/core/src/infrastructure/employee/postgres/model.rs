use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::{
    Employee, EmployeeCostBasis, EmployeeCostBasisId, EmployeeId, MemberId, OrganizationId,
};

#[derive(Debug, Clone)]
pub struct EmployeeRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub hourly_rate_cents: Option<i32>,
    pub is_salaried: bool,
    pub monthly_cost_cents: Option<i32>,
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
            monthly_cost_cents: row.monthly_cost_cents,
            weekly_contract_minutes: row.weekly_contract_minutes,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmployeeCostBasisRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub employee_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub is_salaried: bool,
    pub hourly_rate_cents: Option<i32>,
    pub monthly_cost_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EmployeeCostBasisRow> for EmployeeCostBasis {
    fn from(row: EmployeeCostBasisRow) -> Self {
        Self {
            id: EmployeeCostBasisId(row.id),
            organization_id: OrganizationId(row.org_id),
            employee_id: EmployeeId(row.employee_id),
            effective_from: row.effective_from,
            effective_to: row.effective_to,
            is_salaried: row.is_salaried,
            hourly_rate_cents: row.hourly_rate_cents,
            monthly_cost_cents: row.monthly_cost_cents,
            weekly_contract_minutes: row.weekly_contract_minutes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
