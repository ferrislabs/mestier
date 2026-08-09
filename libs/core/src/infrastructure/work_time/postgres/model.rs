use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::{
    EmployeeId, EmployeeRhythm, EmployeeRhythmId, MemberId, OrganizationId, RhythmSlot,
    RhythmSlotId, WorkSlot, WorkSlotId,
};

#[derive(Debug, Clone)]
pub struct RhythmRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub employee_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RhythmRow {
    pub fn into_employee_rhythm(self, slots: Vec<RhythmSlot>) -> EmployeeRhythm {
        EmployeeRhythm {
            id: EmployeeRhythmId(self.id),
            organization_id: OrganizationId(self.org_id),
            employee_id: EmployeeId(self.employee_id),
            effective_from: self.effective_from,
            effective_to: self.effective_to,
            slots,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RhythmSlotRow {
    pub id: Uuid,
    pub rhythm_id: Uuid,
    pub weekday: i16,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

impl From<RhythmSlotRow> for RhythmSlot {
    fn from(row: RhythmSlotRow) -> Self {
        Self {
            id: RhythmSlotId(row.id),
            rhythm_id: EmployeeRhythmId(row.rhythm_id),
            weekday: row.weekday,
            starts_minute: row.starts_minute,
            ends_minute: row.ends_minute,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkSlotRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub work_date: NaiveDate,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

impl From<WorkSlotRow> for WorkSlot {
    fn from(row: WorkSlotRow) -> Self {
        Self {
            id: WorkSlotId(row.id),
            organization_id: OrganizationId(row.org_id),
            member_id: MemberId(row.member_id),
            work_date: row.work_date,
            starts_minute: row.starts_minute,
            ends_minute: row.ends_minute,
        }
    }
}
