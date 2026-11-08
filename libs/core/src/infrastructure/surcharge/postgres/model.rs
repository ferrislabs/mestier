use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{OrganizationId, RateBp, Surcharge, SurchargeId, SurchargeKind, SurchargeName};

#[derive(Debug, Clone)]
pub struct SurchargeRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub rate_bp: i32,
    pub kind: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

fn corrupt(id: Uuid, column: &str) -> CoreError {
    CoreError::Internal(format!("surcharges {id}: invalid {column}"))
}

impl TryFrom<SurchargeRow> for Surcharge {
    type Error = CoreError;

    fn try_from(row: SurchargeRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: SurchargeId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: SurchargeName::try_from(row.name.as_str())
                .map_err(|_| corrupt(row.id, "name"))?,
            rate: RateBp::try_from(row.rate_bp).map_err(|_| corrupt(row.id, "rate_bp"))?,
            kind: row
                .kind
                .parse::<SurchargeKind>()
                .map_err(|()| corrupt(row.id, "kind"))?,
            archived_at: row.archived_at,
            created_at: row.created_at,
        })
    }
}
