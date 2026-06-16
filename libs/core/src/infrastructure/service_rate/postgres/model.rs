use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{OrganizationId, ServiceRate, ServiceRateId, ServiceRateUnit};

#[derive(Debug, Clone)]
pub struct ServiceRateRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub label: String,
    pub unit: String,
    pub rate_cents: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ServiceRateRow> for ServiceRate {
    type Error = CoreError;

    fn try_from(row: ServiceRateRow) -> Result<Self, Self::Error> {
        let unit = ServiceRateUnit::from_str(&row.unit).map_err(|e| {
            CoreError::Internal(format!("invalid service rate unit in database: {e}"))
        })?;

        Ok(Self {
            id: ServiceRateId(row.id),
            organization_id: OrganizationId(row.org_id),
            label: row.label,
            unit,
            rate_cents: row.rate_cents,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
