use chrono::NaiveDate;

use crate::{
    Cadence, ContractStatus, CustomerContextId, CustomerId, InterventionId, IsoWeekNumber,
    MaintenanceContractId, MaintenanceLineId, MemberId, OrganizationId, QuoteId, Season, TaskId,
};

/// One line as the form sends it. `id` is `None` for a new line.
#[derive(Debug, Clone)]
pub struct MaintenanceLineInput {
    pub id: Option<MaintenanceLineId>,
    pub label: String,
    pub cadence: Cadence,
    pub minutes_per_intervention: Option<u32>,
}

/// Also creates the contract's project, named after the customer, the site
/// and the season.
#[derive(Debug, Clone)]
pub struct CreateMaintenanceContractCommand {
    /// Authenticated actor. Built by the handler from the request `Identity`.
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub reference: Option<String>,
    pub season: Season,
    pub manager_member_id: Option<MemberId>,
    pub amount_cents: Option<i64>,
    pub quote_id: Option<QuoteId>,
    pub notes: Option<String>,
    pub status: ContractStatus,
    pub lines: Vec<MaintenanceLineInput>,
}

/// The form saves the whole contract at once. The customer, the site and
/// the year are fixed at creation.
#[derive(Debug, Clone)]
pub struct UpdateMaintenanceContractCommand {
    pub actor: authz::Subject,
    pub id: MaintenanceContractId,
    pub reference: Option<String>,
    pub season: Season,
    pub manager_member_id: Option<MemberId>,
    pub amount_cents: Option<i64>,
    pub quote_id: Option<QuoteId>,
    pub notes: Option<String>,
    pub status: ContractStatus,
    pub lines: Vec<MaintenanceLineInput>,
}

#[derive(Debug, Clone)]
pub struct ArchiveMaintenanceContractCommand {
    pub actor: authz::Subject,
    pub id: MaintenanceContractId,
}

/// Copies the contract and its lines into the next season, with a new
/// project and no intervention.
#[derive(Debug, Clone)]
pub struct DuplicateMaintenanceContractCommand {
    pub actor: authz::Subject,
    pub id: MaintenanceContractId,
}

/// A manager's direct entry, validated at once.
#[derive(Debug, Clone)]
pub struct RecordInterventionCommand {
    pub actor: authz::Subject,
    pub line_id: MaintenanceLineId,
    pub done_on: NaiveDate,
}

/// A field worker's entry on one of their own maintenance tasks.
#[derive(Debug, Clone)]
pub struct ProposeInterventionCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub done_on: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct ValidateInterventionCommand {
    pub actor: authz::Subject,
    pub id: InterventionId,
}

#[derive(Debug, Clone)]
pub struct RejectInterventionCommand {
    pub actor: authz::Subject,
    pub id: InterventionId,
    pub note: Option<String>,
}

/// Creates one undated backlog task per line in the contract's project,
/// carrying the line and the site.
#[derive(Debug, Clone)]
pub struct PlanMaintenanceLinesCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub line_ids: Vec<MaintenanceLineId>,
    pub year: i32,
    pub week: IsoWeekNumber,
}
