use chrono::{DateTime, Utc};
use mestier_core::{
    ContractStatus, CustomerContextId, CustomerId, MaintenanceContract, MaintenanceContractId,
    MaintenanceLine, MemberId, ProjectId, QuoteId, Season,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct MaintenanceContractResponse {
    pub id: MaintenanceContractId,
    pub customer_id: CustomerId,
    /// The site.
    pub customer_context_id: CustomerContextId,
    /// Created with the contract: the interventions' tasks live there.
    pub project_id: ProjectId,
    pub reference: Option<String>,
    pub season: Season,
    pub manager_member_id: Option<MemberId>,
    pub amount_cents: Option<i64>,
    pub quote_id: Option<QuoteId>,
    pub notes: Option<String>,
    pub status: ContractStatus,
    /// In display order.
    pub lines: Vec<MaintenanceLine>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MaintenanceContract> for MaintenanceContractResponse {
    fn from(contract: MaintenanceContract) -> Self {
        Self {
            id: contract.id,
            customer_id: contract.customer_id,
            customer_context_id: contract.customer_context_id,
            project_id: contract.project_id,
            reference: contract.reference,
            season: contract.season,
            manager_member_id: contract.manager_member_id,
            amount_cents: contract.amount_cents,
            quote_id: contract.quote_id,
            notes: contract.notes,
            status: contract.status,
            lines: contract.lines,
            archived_at: contract.archived_at,
            created_at: contract.created_at,
            updated_at: contract.updated_at,
        }
    }
}
