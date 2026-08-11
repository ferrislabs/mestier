use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{Absence, AbsenceId, AbsenceKind, MemberId, OrganizationId};

#[derive(Debug, Clone)]
pub struct AbsenceRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub kind: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub note: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AbsenceRow {
    pub fn into_employee_absence(self) -> Result<Absence, CoreError> {
        let kind = AbsenceKind::from_str(&self.kind)
            .map_err(|e| CoreError::Internal(format!("invalid absence kind in database: {e}")))?;

        Ok(Absence {
            id: AbsenceId(self.id),
            organization_id: OrganizationId(self.org_id),
            member_id: MemberId(self.member_id),
            kind,
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            all_day: self.all_day,
            note: self.note,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
