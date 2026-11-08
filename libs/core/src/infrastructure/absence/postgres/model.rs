use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    Absence, AbsenceId, AbsenceKind, AbsenceStatus, AbsenceTypeId, DayPart, MemberId,
    OrganizationId,
};

#[derive(Debug, Clone)]
pub struct AbsenceRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub kind: String,
    pub type_id: Uuid,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub start_date: NaiveDate,
    pub start_part: String,
    pub end_date: NaiveDate,
    pub end_part: String,
    pub status: String,
    pub note: Option<String>,
    pub proof_file_key: Option<String>,
    pub created_by_member_id: Option<Uuid>,
    pub decided_by_member_id: Option<Uuid>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_comment: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn corrupt(id: Uuid, column: &str) -> CoreError {
    CoreError::Internal(format!("absences {id}: invalid {column}"))
}

impl AbsenceRow {
    pub fn into_employee_absence(self) -> Result<Absence, CoreError> {
        let kind = AbsenceKind::from_str(&self.kind)
            .map_err(|e| CoreError::Internal(format!("invalid absence kind in database: {e}")))?;
        let start_part =
            DayPart::from_str(&self.start_part).map_err(|()| corrupt(self.id, "start_part"))?;
        let end_part =
            DayPart::from_str(&self.end_part).map_err(|()| corrupt(self.id, "end_part"))?;
        let status =
            AbsenceStatus::from_str(&self.status).map_err(|()| corrupt(self.id, "status"))?;

        Ok(Absence {
            id: AbsenceId(self.id),
            organization_id: OrganizationId(self.org_id),
            member_id: MemberId(self.member_id),
            kind,
            type_id: AbsenceTypeId(self.type_id),
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            all_day: self.all_day,
            start_date: self.start_date,
            start_part,
            end_date: self.end_date,
            end_part,
            status,
            note: self.note,
            proof_file_key: self.proof_file_key,
            created_by_member_id: self.created_by_member_id.map(MemberId),
            decided_by_member_id: self.decided_by_member_id.map(MemberId),
            decided_at: self.decided_at,
            decision_comment: self.decision_comment,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
