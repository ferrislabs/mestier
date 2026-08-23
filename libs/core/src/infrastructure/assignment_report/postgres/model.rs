use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    AssignmentReport, AssignmentReportId, AssignmentReportResolution, MemberId, OrganizationId,
    TaskAssignmentId,
};

#[derive(Debug, Clone)]
pub struct AssignmentReportRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub task_assignment_id: Uuid,
    pub reported_minutes: i32,
    pub comment: Option<String>,
    pub reported_by: Uuid,
    pub resolution: String,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<AssignmentReportRow> for AssignmentReport {
    type Error = CoreError;

    fn try_from(row: AssignmentReportRow) -> Result<Self, Self::Error> {
        let resolution = AssignmentReportResolution::from_str(&row.resolution).map_err(|e| {
            CoreError::Internal(format!(
                "invalid assignment report resolution in database: {e}"
            ))
        })?;

        // `reported_minutes` is `CHECK (>= 0)` in the database, so this can
        // only fail on data corruption — treated as an internal error rather
        // than propagated as a domain-shaped `CoreError`.
        let reported_minutes = u32::try_from(row.reported_minutes).map_err(|_| {
            CoreError::Internal(format!(
                "negative reported_minutes ({}) in database despite the CHECK constraint",
                row.reported_minutes
            ))
        })?;

        Ok(Self {
            id: AssignmentReportId(row.id),
            organization_id: OrganizationId(row.org_id),
            task_assignment_id: TaskAssignmentId(row.task_assignment_id),
            reported_minutes,
            comment: row.comment,
            reported_by: MemberId(row.reported_by),
            resolution,
            resolved_by: row.resolved_by.map(MemberId),
            resolved_at: row.resolved_at,
            resolution_note: row.resolution_note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
