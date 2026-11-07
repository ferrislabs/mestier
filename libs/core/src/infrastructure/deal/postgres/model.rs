use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    CustomerId, Deal, DealAmount, DealId, DealStage, DealStageName, DealTitle, LeadSource,
    MemberId, OrganizationId, StageTotal,
};

#[derive(Debug, Clone)]
pub struct DealRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub customer_id: Uuid,
    pub title: String,
    pub amount_cents: i64,
    pub stage: String,
    pub expected_close_date: Option<NaiveDate>,
    pub owner_member_id: Option<Uuid>,
    pub source: Option<String>,
    pub lost_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub stage_changed_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub quote_count: i64,
}

fn corrupt(what: &str, error: impl std::fmt::Display) -> CoreError {
    CoreError::Internal(format!("invalid deal {what} in database: {error}"))
}

impl TryFrom<DealRow> for Deal {
    type Error = CoreError;

    fn try_from(row: DealRow) -> Result<Self, Self::Error> {
        let name = DealStageName::from_str(&row.stage).map_err(|e| corrupt("stage", e))?;
        let stage = DealStage::try_from((name, row.lost_reason.as_deref()))
            .map_err(|e| corrupt("lost reason", e))?;
        let source = row
            .source
            .as_deref()
            .map(LeadSource::from_str)
            .transpose()
            .map_err(|e| corrupt("source", e))?;

        Ok(Self {
            id: DealId(row.id),
            organization_id: OrganizationId(row.org_id),
            customer_id: CustomerId(row.customer_id),
            title: DealTitle::try_from(row.title).map_err(|e| corrupt("title", e))?,
            amount: DealAmount::try_from(row.amount_cents).map_err(|e| corrupt("amount", e))?,
            stage,
            expected_close_date: row.expected_close_date,
            owner_member_id: row.owner_member_id.map(MemberId),
            source,
            notes: row.notes,
            created_at: row.created_at,
            stage_changed_at: row.stage_changed_at,
            closed_at: row.closed_at,
            quote_count: row.quote_count,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StageTotalRow {
    pub stage: String,
    pub count: i64,
    pub amount_cents: i64,
}

impl TryFrom<StageTotalRow> for StageTotal {
    type Error = CoreError;

    fn try_from(row: StageTotalRow) -> Result<Self, Self::Error> {
        Ok(Self {
            stage: DealStageName::from_str(&row.stage).map_err(|e| corrupt("stage", e))?,
            count: row.count,
            amount_cents: row.amount_cents,
        })
    }
}
