use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    AdjustmentDays, AdjustmentKind, AdjustmentReason, LeaveAccountId, LeaveAdjustment,
    LeaveAdjustmentId, MemberId, OrganizationId,
};

#[derive(Debug, Clone)]
pub struct LeaveAdjustmentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub account_id: Uuid,
    pub date: NaiveDate,
    pub days_hundredths: i32,
    pub kind: String,
    pub reason: String,
    pub author_member_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

fn corrupt(id: Uuid, column: &str) -> CoreError {
    CoreError::Internal(format!("leave_adjustments {id}: invalid {column}"))
}

impl TryFrom<LeaveAdjustmentRow> for LeaveAdjustment {
    type Error = CoreError;

    fn try_from(row: LeaveAdjustmentRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: LeaveAdjustmentId(row.id),
            organization_id: OrganizationId(row.org_id),
            member_id: MemberId(row.member_id),
            account_id: LeaveAccountId(row.account_id),
            date: row.date,
            days: AdjustmentDays::from_hundredths(row.days_hundredths)
                .ok_or_else(|| corrupt(row.id, "days_hundredths"))?,
            kind: row
                .kind
                .parse::<AdjustmentKind>()
                .map_err(|()| corrupt(row.id, "kind"))?,
            reason: AdjustmentReason::try_from(row.reason.as_str())
                .map_err(|_| corrupt(row.id, "reason"))?,
            author_member_id: row.author_member_id.map(MemberId),
            created_at: row.created_at,
        })
    }
}
