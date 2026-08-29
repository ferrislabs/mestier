use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    OrganizationId, ProjectId, ProjectSupplierCostLine, SupplierId, SupplierInvoice,
    SupplierInvoiceId, SupplierInvoiceLine, SupplierInvoiceLineAllocation,
    SupplierInvoiceLineAllocationId, SupplierInvoiceLineId, SupplierInvoiceSource,
    SupplierInvoiceStatus, SupplierInvoiceVatBreakdownLine,
    application::supplier_invoice::TotalsMismatch,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct SupplierInvoiceLineResponse {
    pub id: SupplierInvoiceLineId,
    pub supplier_invoice_id: SupplierInvoiceId,
    pub label: String,
    pub quantity: String,
    pub unit: Option<String>,
    pub unit_price_cents: i32,
    pub line_total_cents: i32,
    pub vat_rate_basis_points: Option<i32>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SupplierInvoiceLine> for SupplierInvoiceLineResponse {
    fn from(value: SupplierInvoiceLine) -> Self {
        Self {
            id: value.id,
            supplier_invoice_id: value.supplier_invoice_id,
            label: value.label,
            quantity: value.quantity.normalize().to_string(),
            unit: value.unit,
            unit_price_cents: value.unit_price_cents,
            line_total_cents: value.line_total_cents,
            vat_rate_basis_points: value.vat_rate_basis_points,
            position: value.position,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct SupplierInvoiceVatBreakdownLineResponse {
    pub rate_bp: i32,
    pub vat_cents: i32,
}

impl From<SupplierInvoiceVatBreakdownLine> for SupplierInvoiceVatBreakdownLineResponse {
    fn from(value: SupplierInvoiceVatBreakdownLine) -> Self {
        Self {
            rate_bp: value.rate_bp,
            vat_cents: value.vat_cents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct SupplierInvoiceResponse {
    pub id: SupplierInvoiceId,
    pub organization_id: OrganizationId,
    pub supplier_id: Option<SupplierId>,
    pub supplier_name: String,
    pub supplier_registration_number: Option<String>,
    pub supplier_vat_number: Option<String>,
    pub number: String,
    pub issued_on: NaiveDate,
    pub due_on: Option<NaiveDate>,
    pub received_at: DateTime<Utc>,
    pub source: SupplierInvoiceSource,
    pub status: SupplierInvoiceStatus,
    pub currency: String,
    /// A time-limited URL the browser can fetch the original file from
    /// directly is a separate call (`GET /api/v1/files/{key}/url`,
    /// `handlers-files`) — this is the key it takes, `None` when the
    /// invoice was entered by hand with no file behind it at all.
    pub source_file_key: Option<String>,
    pub source_file_mime_type: Option<String>,
    pub notes: Option<String>,
    pub net_cents: i32,
    pub vat_breakdown: Vec<SupplierInvoiceVatBreakdownLineResponse>,
    pub gross_cents: i32,
    pub lines: Vec<SupplierInvoiceLineResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SupplierInvoice> for SupplierInvoiceResponse {
    fn from(value: SupplierInvoice) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            supplier_id: value.supplier_id,
            supplier_name: value.supplier_name,
            supplier_registration_number: value.supplier_registration_number,
            supplier_vat_number: value.supplier_vat_number,
            number: value.number,
            issued_on: value.issued_on,
            due_on: value.due_on,
            received_at: value.received_at,
            source: value.source,
            status: value.status,
            currency: value.currency,
            source_file_key: value.source_file_key,
            source_file_mime_type: value.source_file_mime_type,
            notes: value.notes,
            net_cents: value.net_cents,
            vat_breakdown: value
                .vat_breakdown
                .into_iter()
                .map(SupplierInvoiceVatBreakdownLineResponse::from)
                .collect(),
            gross_cents: value.gross_cents,
            lines: value
                .lines
                .into_iter()
                .map(SupplierInvoiceLineResponse::from)
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// A stated-vs-recomputed totals disagreement (#337) — surfaced on the
/// import response, never silently resolved either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct TotalsMismatchResponse {
    pub stated_net_cents: i32,
    pub recomputed_net_cents: i32,
    pub stated_gross_cents: i32,
    pub recomputed_gross_cents: i32,
}

impl From<TotalsMismatch> for TotalsMismatchResponse {
    fn from(value: TotalsMismatch) -> Self {
        Self {
            stated_net_cents: value.stated_net_cents,
            recomputed_net_cents: value.recomputed_net_cents,
            stated_gross_cents: value.stated_gross_cents,
            recomputed_gross_cents: value.recomputed_gross_cents,
        }
    }
}

/// What `POST .../supplier-invoices/import` answers with — a file that
/// fails to parse is a legitimate outcome (#337's binding rule: "kept,
/// with the reason"), not a 4xx/5xx, so the response has to say which of
/// the two actually happened rather than always shaping itself like a
/// created invoice.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ImportSupplierInvoiceResponse {
    Created {
        // Boxed for the same reason `ImportSupplierInvoiceOutcome::Created`
        // boxes its own invoice: `ParseFailed` should not pay this
        // variant's size just by existing alongside it.
        invoice: Box<SupplierInvoiceResponse>,
        totals_mismatch: Option<TotalsMismatchResponse>,
    },
    ParseFailed {
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct SupplierInvoiceLineAllocationResponse {
    pub id: SupplierInvoiceLineAllocationId,
    pub organization_id: OrganizationId,
    pub supplier_invoice_line_id: SupplierInvoiceLineId,
    pub project_id: ProjectId,
    pub amount_cents: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SupplierInvoiceLineAllocation> for SupplierInvoiceLineAllocationResponse {
    fn from(value: SupplierInvoiceLineAllocation) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            supplier_invoice_line_id: value.supplier_invoice_line_id,
            project_id: value.project_id,
            amount_cents: value.amount_cents,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// One line of a project's supplier cost, with enough of its parent invoice
/// to link back to it — see [`ProjectSupplierCostLine`]'s own doc comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ProjectSupplierCostLineResponse {
    pub allocation_id: SupplierInvoiceLineAllocationId,
    pub supplier_invoice_id: SupplierInvoiceId,
    pub supplier_invoice_number: String,
    pub supplier_name: String,
    pub supplier_invoice_line_id: SupplierInvoiceLineId,
    pub line_label: String,
    pub amount_cents: i32,
    pub created_at: DateTime<Utc>,
}

impl From<ProjectSupplierCostLine> for ProjectSupplierCostLineResponse {
    fn from(value: ProjectSupplierCostLine) -> Self {
        Self {
            allocation_id: value.allocation_id,
            supplier_invoice_id: value.supplier_invoice_id,
            supplier_invoice_number: value.supplier_invoice_number,
            supplier_name: value.supplier_name,
            supplier_invoice_line_id: value.supplier_invoice_line_id,
            line_label: value.line_label,
            amount_cents: value.amount_cents,
            created_at: value.created_at,
        }
    }
}

/// What a project's supplier costs look like on its own screen — the plain
/// net sum `SupplierInvoiceService::allocated_cost_for_project` computes,
/// deliberately not the same figure the profitability report states (see
/// that method's own doc comment: this one is unfiltered by status or
/// period, the report's is not). `lines` is the same total, itemized: #340
/// links each cost back to the invoice it came from.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProjectSupplierCostsResponse {
    pub project_id: ProjectId,
    pub allocated_cents: i64,
    pub lines: Vec<ProjectSupplierCostLineResponse>,
}
