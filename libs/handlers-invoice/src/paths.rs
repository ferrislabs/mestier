use axum_extra::routing::TypedPath;
use mestier_core::{InvoiceId, InvoicePaymentId, OrganizationId, ProjectId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/invoices")]
pub struct OrganizationInvoicesPath {
    pub organization_id: OrganizationId,
}

/// The dunning read #320 built (`outstanding_balance_by_customer`) and #319
/// left unwired — see `invoice::outstanding`.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/invoices/outstanding")]
pub struct OrganizationInvoicesOutstandingPath {
    pub organization_id: OrganizationId,
}

/// Bare — an invoice's organization is derived from the loaded row, never
/// from the path (see `require_invoice_membership`).
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}")]
pub struct InvoicePath {
    pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/issue")]
pub struct InvoiceIssuePath {
    pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/cancel")]
pub struct InvoiceCancelPath {
    pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/credit-notes")]
pub struct InvoiceCreditNotesPath {
    pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/pdf")]
pub struct InvoicePdfPath {
    pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/payments")]
pub struct InvoicePaymentsPath {
    pub invoice_id: InvoiceId,
}

/// What remains to be collected on this invoice — see `invoice::balance`.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/balance")]
pub struct InvoiceBalancePath {
    pub invoice_id: InvoiceId,
}

/// Bare — a payment's organization is derived from the loaded row, via the
/// invoice it belongs to (see `require_payment_membership`).
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoice-payments/{invoice_payment_id}")]
pub struct InvoicePaymentPath {
    pub invoice_payment_id: InvoicePaymentId,
}

/// Bare — a project's organization is derived from the loaded row, never
/// from the path (see `require_project_membership`).
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/projects/{project_id}/invoices")]
pub struct ProjectInvoicesPath {
    pub project_id: ProjectId,
}

/// The two project-scoped acts #317 shipped in the domain with no route to
/// reach them from — `issue_deposit`/`issue_final_invoice` — added here
/// because leaving them unreachable would make #322 ("invoice a project:
/// everything remaining, a percentage, or a free amount") impossible to
/// build later. Not in the master issue's own route list.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/projects/{project_id}/invoices/deposit")]
pub struct ProjectInvoiceDepositPath {
    pub project_id: ProjectId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/projects/{project_id}/invoices/final")]
pub struct ProjectInvoiceFinalPath {
    pub project_id: ProjectId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/projects/{project_id}/billing-summary")]
pub struct ProjectBillingSummaryPath {
    pub project_id: ProjectId,
}
