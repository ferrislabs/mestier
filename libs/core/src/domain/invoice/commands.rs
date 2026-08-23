use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::{
    CustomerContextId, CustomerId, InvoiceId, InvoiceKind, OperationNature, OrganizationAddress,
    OrganizationId, ProjectId,
};

/// Issues any draft invoice, whatever `InvoiceKind` built it — a manually
/// created `Standard` draft from #316's `create_invoice`, or a
/// `Deposit`/`Final` draft from this issue's own `issue_deposit`/
/// `issue_final_invoice`. See `InvoiceService::issue_invoice`.
#[derive(Debug, Clone, Copy)]
pub struct IssueInvoiceCommand {
    pub id: InvoiceId,
    /// Refuses issuing when it would push the project's issued total past
    /// the quote, unless the caller explicitly says this is fine: a
    /// deliberate over-bill happens, the artisan should not have to lie to
    /// the system to do it, but the system should not do it silently
    /// either.
    pub allow_exceeding_total: bool,
}

/// Builds a deposit invoice for a percentage of the project's quoted total
/// and issues it in one step. Needs a quote to compute a percentage
/// against — a general limitation of this specific act, not of invoicing
/// as a whole (`create_invoice`/`issue_invoice` still work with no
/// project, or a project with no quote).
#[derive(Debug, Clone)]
pub struct IssueDepositCommand {
    pub project_id: ProjectId,
    /// Basis points of the quote's net total, e.g. 3000 = 30%.
    pub percentage_bp: i32,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub allow_exceeding_total: bool,
}

/// Builds an invoice for exactly what remains to bill on the project's
/// quote — `remaining_to_bill_cents` after every already-issued,
/// non-cancelled invoice — and issues it in one step. Same quote
/// requirement, and the same reason, as `IssueDepositCommand`.
#[derive(Debug, Clone)]
pub struct IssueFinalInvoiceCommand {
    pub project_id: ProjectId,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub allow_exceeding_total: bool,
}

#[derive(Debug, Clone)]
pub struct InvoiceLineCommand {
    pub label: String,
    pub quantity: Decimal,
    pub unit_price_cents: i32,
    pub vat_rate_basis_points: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct CreateInvoiceCommand {
    pub organization_id: OrganizationId,
    pub kind: InvoiceKind,
    pub project_id: Option<ProjectId>,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub operation_nature: Option<OperationNature>,
    pub delivery_address: Option<OrganizationAddress>,
    pub lines: Vec<InvoiceLineCommand>,
}

#[derive(Debug, Clone)]
pub struct UpdateInvoiceCommand {
    pub id: InvoiceId,
    pub project_id: Option<ProjectId>,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub operation_nature: Option<OperationNature>,
    pub delivery_address: Option<OrganizationAddress>,
    pub lines: Vec<InvoiceLineCommand>,
}

/// Cancels an invoice, draft or issued. Never used to reach `Issued`,
/// `Paid` or `PartiallyPaid`: those are set by issuing (#317) and by
/// recording payments (#320), never by hand.
#[derive(Debug, Clone, Copy)]
pub struct CancelInvoiceCommand {
    pub id: InvoiceId,
}
