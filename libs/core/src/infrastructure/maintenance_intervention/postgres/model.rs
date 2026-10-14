use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    Intervention, InterventionId, InterventionStatus, MaintenanceLineId, MemberId, OrganizationId,
    TaskId,
};

#[derive(Debug, Clone)]
pub struct InterventionRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub line_id: Uuid,
    pub done_on: NaiveDate,
    pub status: String,
    pub task_id: Option<Uuid>,
    pub proposed_by: Uuid,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<InterventionRow> for Intervention {
    type Error = CoreError;

    fn try_from(row: InterventionRow) -> Result<Self, Self::Error> {
        let status = row
            .status
            .parse::<InterventionStatus>()
            .map_err(CoreError::Internal)?;

        Ok(Self {
            id: InterventionId(row.id),
            organization_id: OrganizationId(row.org_id),
            line_id: MaintenanceLineId(row.line_id),
            done_on: row.done_on,
            status,
            task_id: row.task_id.map(TaskId),
            proposed_by: MemberId(row.proposed_by),
            resolved_by: row.resolved_by.map(MemberId),
            resolved_at: row.resolved_at,
            note: row.note,
            created_at: row.created_at,
        })
    }
}
