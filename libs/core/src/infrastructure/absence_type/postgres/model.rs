use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    AbsenceKind, AbsenceType, AbsenceTypeId, AbsenceTypeName, CountMode, Hue, OrganizationId,
};

#[derive(Debug, Clone)]
pub struct AbsenceTypeRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub hue: i16,
    pub count_mode: String,
    pub account_id: Option<Uuid>,
    pub paid: bool,
    pub needs_approval: bool,
    pub needs_proof: bool,
    pub kind: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

fn corrupt(id: Uuid, column: &str) -> CoreError {
    CoreError::Internal(format!("absence_types {id}: invalid {column}"))
}

impl TryFrom<AbsenceTypeRow> for AbsenceType {
    type Error = CoreError;

    fn try_from(row: AbsenceTypeRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: AbsenceTypeId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: AbsenceTypeName::try_from(row.name.as_str())
                .map_err(|_| corrupt(row.id, "name"))?,
            hue: u16::try_from(row.hue)
                .ok()
                .and_then(|hue| Hue::try_from(hue).ok())
                .ok_or_else(|| corrupt(row.id, "hue"))?,
            count_mode: row
                .count_mode
                .parse::<CountMode>()
                .map_err(|()| corrupt(row.id, "count_mode"))?,
            account_id: row.account_id,
            paid: row.paid,
            needs_approval: row.needs_approval,
            needs_proof: row.needs_proof,
            kind: row
                .kind
                .parse::<AbsenceKind>()
                .map_err(|_| corrupt(row.id, "kind"))?,
            archived_at: row.archived_at,
            created_at: row.created_at,
        })
    }
}
