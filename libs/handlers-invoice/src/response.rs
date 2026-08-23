use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    CustomerContextId, CustomerId, CustomerOutstandingBalance, Invoice, InvoiceId, InvoiceKind,
    InvoiceLine, InvoiceLineId, InvoicePayment, InvoicePaymentId, InvoiceStatus,
    InvoiceVatBreakdownLine, OperationNature, OrganizationAddress, OrganizationId,
    ProjectBillingSummary, ProjectId, UserId,
};
use serde::Serialize;
use utoipa::ToSchema;

/// Where the goods or services were delivered, when it differs from the
/// customer's own address — the fourth e-invoicing mention #341 adds.
/// `OrganizationAddress` itself carries no `Serialize`/`ToSchema` (it is a
/// domain type, built only through `LegalIdentity`), so this is a small
/// mirror rather than a re-export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct DeliveryAddressResponse {
    pub line1: String,
    pub line2: Option<String>,
    pub postal_code: String,
    pub city: String,
    pub country: String,
}

impl From<OrganizationAddress> for DeliveryAddressResponse {
    fn from(value: OrganizationAddress) -> Self {
        Self {
            line1: value.line1,
            line2: value.line2,
            postal_code: value.postal_code,
            city: value.city,
            country: value.country,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct InvoiceLineResponse {
    pub id: InvoiceLineId,
    pub organization_id: OrganizationId,
    pub invoice_id: InvoiceId,
    pub label: String,
    pub quantity: String,
    pub unit_price_cents: i32,
    pub vat_rate_basis_points: Option<i32>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<InvoiceLine> for InvoiceLineResponse {
    fn from(value: InvoiceLine) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            invoice_id: value.invoice_id,
            label: value.label,
            quantity: value.quantity.normalize().to_string(),
            unit_price_cents: value.unit_price_cents,
            vat_rate_basis_points: value.vat_rate_basis_points,
            position: value.position,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct InvoiceVatBreakdownLineResponse {
    pub rate_bp: i32,
    pub vat_cents: i32,
}

impl From<InvoiceVatBreakdownLine> for InvoiceVatBreakdownLineResponse {
    fn from(value: InvoiceVatBreakdownLine) -> Self {
        Self {
            rate_bp: value.rate_bp,
            vat_cents: value.vat_cents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct InvoiceResponse {
    pub id: InvoiceId,
    pub organization_id: OrganizationId,
    /// `None` on a draft: a number is allocated exactly when the invoice
    /// leaves `Draft`, never before.
    pub number: Option<String>,
    pub kind: InvoiceKind,
    pub project_id: Option<ProjectId>,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    /// `PAID`/`PARTIALLY_PAID` are decorated on read, never persisted as
    /// such — see `InvoiceService::decorate_status`. This field can read
    /// differently across two calls with nothing written in between, by
    /// design.
    pub status: InvoiceStatus,
    pub issued_at: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub operation_nature: Option<OperationNature>,
    pub delivery_address: Option<DeliveryAddressResponse>,
    pub net_cents: i32,
    pub vat_breakdown: Vec<InvoiceVatBreakdownLineResponse>,
    pub gross_cents: i32,
    pub lines: Vec<InvoiceLineResponse>,
    /// Set exactly when `kind` is `CREDIT_NOTE` — the invoice this one
    /// corrects.
    pub source_invoice_id: Option<InvoiceId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Invoice> for InvoiceResponse {
    fn from(value: Invoice) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            number: value.number,
            kind: value.kind,
            project_id: value.project_id,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            status: value.status,
            issued_at: value.issued_at,
            due_at: value.due_at,
            notes: value.notes,
            operation_nature: value.operation_nature,
            delivery_address: value.delivery_address.map(DeliveryAddressResponse::from),
            net_cents: value.net_cents,
            vat_breakdown: value
                .vat_breakdown
                .into_iter()
                .map(InvoiceVatBreakdownLineResponse::from)
                .collect(),
            gross_cents: value.gross_cents,
            lines: value
                .lines
                .into_iter()
                .map(InvoiceLineResponse::from)
                .collect(),
            source_invoice_id: value.source_invoice_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct InvoicePaymentResponse {
    pub id: InvoicePaymentId,
    pub organization_id: OrganizationId,
    pub invoice_id: InvoiceId,
    pub amount_cents: i32,
    pub paid_on: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub recorded_by: UserId,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<InvoicePayment> for InvoicePaymentResponse {
    fn from(value: InvoicePayment) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            invoice_id: value.invoice_id,
            amount_cents: value.amount_cents,
            paid_on: value.paid_on,
            method: value.method,
            reference: value.reference,
            note: value.note,
            recorded_by: value.recorded_by,
            deleted_at: value.deleted_at,
            deleted_by: value.deleted_by,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// Mirrors `CustomerOutstandingBalance`. No route reads this yet — see the
/// commit message: only `issue_deposit`/`issue_final_invoice` were named as
/// unreachable domain capabilities to wire up in this issue, the dunning
/// list (`outstanding_balance_by_customer`) is left for its own follow-up —
/// but the response shape is defined alongside the rest of this crate's DTOs
/// so that follow-up is a route, not a type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct CustomerOutstandingBalanceResponse {
    pub customer_id: CustomerId,
    pub outstanding_cents: i64,
    pub oldest_due_at: Option<DateTime<Utc>>,
}

impl From<CustomerOutstandingBalance> for CustomerOutstandingBalanceResponse {
    fn from(value: CustomerOutstandingBalance) -> Self {
        Self {
            customer_id: value.customer_id,
            outstanding_cents: value.outstanding_cents,
            oldest_due_at: value.oldest_due_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct ProjectBillingSummaryResponse {
    pub project_id: ProjectId,
    pub quoted_cents: Option<i32>,
    pub billed_cents: i64,
    pub remaining_cents: Option<i64>,
}

impl From<ProjectBillingSummary> for ProjectBillingSummaryResponse {
    fn from(value: ProjectBillingSummary) -> Self {
        Self {
            project_id: value.project_id,
            quoted_cents: value.quoted_cents,
            billed_cents: value.billed_cents,
            remaining_cents: value.remaining_cents,
        }
    }
}
