use chrono::{DateTime, Utc};
use mestier_core::{
    CustomerContextId, CustomerId, OrganizationId, Project, ProjectId, Quote, QuoteId, QuoteLine,
    QuoteLineId, QuoteStatus, QuoteVatBreakdownLine, ServiceRateId, ServiceRateUnit, Task, TaskId,
    TaskProposal,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct QuoteLineResponse {
    pub id: QuoteLineId,
    pub organization_id: OrganizationId,
    pub quote_id: QuoteId,
    pub service_rate_id: Option<ServiceRateId>,
    pub label: String,
    pub quantity: String,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub vat_rate_bp: Option<i32>,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<QuoteLine> for QuoteLineResponse {
    fn from(value: QuoteLine) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            quote_id: value.quote_id,
            service_rate_id: value.service_rate_id,
            label: value.label,
            quantity: value.quantity.normalize().to_string(),
            unit: value.unit,
            unit_price_cents: value.unit_price_cents,
            vat_rate_bp: value.vat_rate_bp,
            notes: value.notes,
            photo_keys: value.photo_keys,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct QuoteVatBreakdownLineResponse {
    pub rate_bp: i32,
    pub vat_cents: i32,
}

impl From<QuoteVatBreakdownLine> for QuoteVatBreakdownLineResponse {
    fn from(value: QuoteVatBreakdownLine) -> Self {
        Self {
            rate_bp: value.rate_bp,
            vat_cents: value.vat_cents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct QuoteResponse {
    pub id: QuoteId,
    pub organization_id: OrganizationId,
    /// `None` on a draft: no number is allocated until the quote is sent.
    pub reference: Option<String>,
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub status: QuoteStatus,
    pub net_cents: i32,
    pub vat_breakdown: Vec<QuoteVatBreakdownLineResponse>,
    pub gross_cents: i32,
    pub lines: Vec<QuoteLineResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Quote> for QuoteResponse {
    fn from(value: Quote) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            reference: value.reference,
            title: value.title,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            status: value.status,
            net_cents: value.net_cents,
            vat_breakdown: value
                .vat_breakdown
                .into_iter()
                .map(QuoteVatBreakdownLineResponse::from)
                .collect(),
            gross_cents: value.gross_cents,
            lines: value
                .lines
                .into_iter()
                .map(QuoteLineResponse::from)
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// One quote line's suggested task, before a human confirms anything — see
/// `mestier_core::TaskProposal`'s own doc comment on why `suggested_minutes`
/// is absent for anything not priced by the hour.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TaskProposalResponse {
    pub quote_line_id: QuoteLineId,
    pub title: String,
    pub suggested_minutes: Option<i32>,
}

impl From<TaskProposal> for TaskProposalResponse {
    fn from(value: TaskProposal) -> Self {
        Self {
            quote_line_id: value.quote_line_id,
            title: value.title,
            suggested_minutes: value.suggested_minutes,
        }
    }
}

/// `GET /quotes/{quote_id}/plan-proposal`'s body: the quote as-is, plus one
/// proposal per line, for a caller to review before confirming a plan.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct QuotePlanProposalResponse {
    pub quote: QuoteResponse,
    pub tasks: Vec<TaskProposalResponse>,
}

/// The project a quote-handover plan produced, as this crate returns it.
///
/// Deliberately not `handlers_planning::response::ProjectResponse`: this
/// crate does not depend on `handlers-planning` (mirrors
/// `TaskEquipmentResponse`'s own doc comment on the same choice), so the
/// handful of fields a confirmation screen needs are duplicated here rather
/// than imported.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PlannedProjectResponse {
    pub id: ProjectId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Project> for PlannedProjectResponse {
    fn from(value: Project) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            quote_id: value.quote_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// A task a quote-handover plan produced. Same duplication reasoning as
/// [`PlannedProjectResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PlannedTaskResponse {
    pub id: TaskId,
    pub parent_task_id: Option<TaskId>,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub project_id: Option<ProjectId>,
    pub expenses_cents: i32,
    pub expenses_label: Option<String>,
}

impl From<Task> for PlannedTaskResponse {
    fn from(value: Task) -> Self {
        Self {
            id: value.id,
            parent_task_id: value.parent_task_id,
            title: value.title,
            description: value.description,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            all_day: value.all_day,
            project_id: value.project_id,
            expenses_cents: value.expenses_cents,
            expenses_label: value.expenses_label,
        }
    }
}

/// `POST /quotes/{quote_id}/plan`'s body: the project the confirmed plan
/// produced, and every task created under it.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct QuotePlanResponse {
    pub project: PlannedProjectResponse,
    pub tasks: Vec<PlannedTaskResponse>,
}
