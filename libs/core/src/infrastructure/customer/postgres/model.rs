use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{Customer, CustomerId, CustomerPipelineStage, CustomerStatus, OrganizationId};

#[derive(Debug, Clone)]
pub struct CustomerRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub status: String,
    pub pipeline_stage: String,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<CustomerRow> for Customer {
    type Error = CoreError;

    fn try_from(row: CustomerRow) -> Result<Self, Self::Error> {
        let status = CustomerStatus::from_str(&row.status).map_err(|e| {
            CoreError::Internal(format!("invalid customer status in database: {e}"))
        })?;
        let pipeline_stage = CustomerPipelineStage::from_str(&row.pipeline_stage).map_err(|e| {
            CoreError::Internal(format!("invalid customer pipeline stage in database: {e}"))
        })?;

        Ok(Self {
            id: CustomerId(row.id),
            organization_id: OrganizationId(row.org_id),
            status,
            pipeline_stage,
            name: row.name,
            phone: row.phone,
            email: row.email,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
