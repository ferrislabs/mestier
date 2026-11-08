use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    Acquisition, CarryOverMax, LeaveAccount, LeaveAccountId, LeaveAccountName, OrganizationId,
    PeriodStartMonth, PositiveDays,
};

#[derive(Debug, Clone)]
pub struct LeaveAccountRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub acquisition_kind: String,
    pub acquisition_hundredths: i32,
    pub period_start_month: i16,
    pub carry_over_max_hundredths: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

fn corrupt(id: Uuid, column: &str) -> CoreError {
    CoreError::Internal(format!("leave_accounts {id}: invalid {column}"))
}

impl TryFrom<LeaveAccountRow> for LeaveAccount {
    type Error = CoreError;

    fn try_from(row: LeaveAccountRow) -> Result<Self, Self::Error> {
        let amount = PositiveDays::from_hundredths(row.acquisition_hundredths)
            .ok_or_else(|| corrupt(row.id, "acquisition_hundredths"))?;
        Ok(Self {
            id: LeaveAccountId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: LeaveAccountName::try_from(row.name.as_str())
                .map_err(|_| corrupt(row.id, "name"))?,
            acquisition: Acquisition::from_parts(&row.acquisition_kind, amount)
                .ok_or_else(|| corrupt(row.id, "acquisition_kind"))?,
            period_start_month: u8::try_from(row.period_start_month)
                .ok()
                .and_then(|month| PeriodStartMonth::try_from(month).ok())
                .ok_or_else(|| corrupt(row.id, "period_start_month"))?,
            carry_over_max: CarryOverMax::from_hundredths(row.carry_over_max_hundredths)
                .ok_or_else(|| corrupt(row.id, "carry_over_max_hundredths"))?,
            archived_at: row.archived_at,
            created_at: row.created_at,
        })
    }
}
