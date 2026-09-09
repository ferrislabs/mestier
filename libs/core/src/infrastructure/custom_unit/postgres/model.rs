use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{CustomUnit, CustomUnitId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CustomUnitRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub code: String,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<CustomUnitRow> for CustomUnit {
    type Error = CoreError;

    fn try_from(row: CustomUnitRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: CustomUnitId(row.id),
            organization_id: OrganizationId(row.org_id),
            code: row.code,
            created_at: row.created_at,
        })
    }
}
