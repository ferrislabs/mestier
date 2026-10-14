use chrono::{DateTime, Utc};
use common::CoreError;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    Cadence, ContractStatus, CustomerContextId, CustomerId, IsoWeekNumber, MaintenanceContract,
    MaintenanceContractId, MaintenanceLine, MaintenanceLineId, MemberId, OrganizationId, ProjectId,
    QuoteId, Season,
};

#[derive(Debug, Clone)]
pub struct ContractRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub customer_id: Uuid,
    pub customer_context_id: Uuid,
    pub project_id: Uuid,
    pub reference: Option<String>,
    pub season_year: i16,
    pub first_week: i16,
    pub last_week: i16,
    pub manager_member_id: Option<Uuid>,
    pub amount_cents: Option<i64>,
    pub quote_id: Option<Uuid>,
    pub notes: Option<String>,
    pub status: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LineRow {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub label: String,
    pub cadence: Value,
    pub minutes_per_intervention: Option<i32>,
}

impl TryFrom<LineRow> for MaintenanceLine {
    type Error = CoreError;

    fn try_from(row: LineRow) -> Result<Self, Self::Error> {
        let cadence: Cadence = serde_json::from_value(row.cadence).map_err(|error| {
            CoreError::Internal(format!(
                "stored cadence of line {} is invalid: {error}",
                row.id
            ))
        })?;
        let minutes_per_intervention = row
            .minutes_per_intervention
            .map(u32::try_from)
            .transpose()
            .map_err(|error| CoreError::Internal(format!("stored minutes are invalid: {error}")))?;

        Ok(Self {
            id: MaintenanceLineId(row.id),
            label: row.label,
            cadence,
            minutes_per_intervention,
        })
    }
}

impl ContractRow {
    pub fn into_contract(
        self,
        lines: Vec<MaintenanceLine>,
    ) -> Result<MaintenanceContract, CoreError> {
        let invalid = |what: &str, error: CoreError| {
            CoreError::Internal(format!(
                "stored contract {} has an invalid {what}: {error}",
                self.id
            ))
        };
        let week = |value: i16| {
            u8::try_from(value)
                .map_err(|_| CoreError::Conflict("week out of range".to_owned()))
                .and_then(IsoWeekNumber::try_from)
        };
        let season = Season::new(
            i32::from(self.season_year),
            week(self.first_week).map_err(|error| invalid("first week", error))?,
            week(self.last_week).map_err(|error| invalid("last week", error))?,
        )
        .map_err(|error| invalid("season", error))?;
        let status = self
            .status
            .parse::<ContractStatus>()
            .map_err(CoreError::Internal)?;

        Ok(MaintenanceContract {
            id: MaintenanceContractId(self.id),
            organization_id: OrganizationId(self.org_id),
            customer_id: CustomerId(self.customer_id),
            customer_context_id: CustomerContextId(self.customer_context_id),
            project_id: ProjectId(self.project_id),
            reference: self.reference,
            season,
            manager_member_id: self.manager_member_id.map(MemberId),
            amount_cents: self.amount_cents,
            quote_id: self.quote_id.map(QuoteId),
            notes: self.notes,
            status,
            lines,
            archived_at: self.archived_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
