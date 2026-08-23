use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::{CustomerContextId, CustomerId, InvoiceId, InvoiceKind, OrganizationId, ProjectId};

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
    pub lines: Vec<InvoiceLineCommand>,
}

/// Cancels an invoice, draft or issued. Never used to reach `Issued`,
/// `Paid` or `PartiallyPaid`: those are set by issuing (#317) and by
/// recording payments (#320), never by hand.
#[derive(Debug, Clone, Copy)]
pub struct CancelInvoiceCommand {
    pub id: InvoiceId,
}
