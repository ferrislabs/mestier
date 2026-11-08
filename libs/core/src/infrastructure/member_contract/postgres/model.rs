use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    ContractPeriod, ContractSchedule, ContractType, MemberContract, MemberContractId, MemberId,
    OrganizationId, ScheduleInput,
};

#[derive(Debug, Clone)]
pub struct MemberContractRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub trial_end_date: Option<NaiveDate>,
    pub schedule: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn corrupt(id: Uuid, what: &str, detail: impl std::fmt::Display) -> CoreError {
    CoreError::Internal(format!("member_contracts {id}: {what}: {detail}"))
}

impl TryFrom<MemberContractRow> for MemberContract {
    type Error = CoreError;

    fn try_from(row: MemberContractRow) -> Result<Self, Self::Error> {
        let contract_type = row
            .contract_type
            .parse::<ContractType>()
            .map_err(|()| corrupt(row.id, "type", &row.contract_type))?;
        let period = ContractPeriod::new(row.start_date, row.end_date, row.trial_end_date)
            .map_err(|error| corrupt(row.id, "period", error))?;
        let input = serde_json::from_value::<ScheduleInput>(row.schedule)
            .map_err(|error| corrupt(row.id, "schedule", error))?;
        let schedule = ContractSchedule::try_from(input)
            .map_err(|error| corrupt(row.id, "schedule", error))?;
        Ok(Self {
            id: MemberContractId(row.id),
            organization_id: OrganizationId(row.org_id),
            member_id: MemberId(row.member_id),
            contract_type,
            period,
            schedule,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
