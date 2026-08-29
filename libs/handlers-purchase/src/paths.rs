use axum_extra::routing::TypedPath;
use mestier_core::{OrganizationId, ProjectId, SupplierInvoiceId, SupplierInvoiceLineId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/supplier-invoices")]
pub struct OrganizationSupplierInvoicesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/supplier-invoices/import")]
pub struct OrganizationSupplierInvoicesImportPath {
    pub organization_id: OrganizationId,
}

/// Bare — a supplier invoice's organization is derived from the loaded
/// row, never from the path (CLAUDE.md: "bare ids derive their
/// organization from the loaded row, never from the path").
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/supplier-invoices/{supplier_invoice_id}")]
pub struct SupplierInvoicePath {
    pub supplier_invoice_id: SupplierInvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/supplier-invoices/{supplier_invoice_id}/confirm")]
pub struct SupplierInvoiceConfirmPath {
    pub supplier_invoice_id: SupplierInvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/supplier-invoices/{supplier_invoice_id}/reject")]
pub struct SupplierInvoiceRejectPath {
    pub supplier_invoice_id: SupplierInvoiceId,
}

/// Bare — same reasoning as [`SupplierInvoicePath`]; a line's organization
/// is derived from the parent invoice loaded alongside it.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/supplier-invoice-lines/{supplier_invoice_line_id}/allocations")]
pub struct SupplierInvoiceLineAllocationsPath {
    pub supplier_invoice_line_id: SupplierInvoiceLineId,
}

/// Bare — a project's organization is derived from the loaded row, mirrors
/// `handlers-invoice`'s `ProjectInvoicesPath`.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/projects/{project_id}/supplier-costs")]
pub struct ProjectSupplierCostsPath {
    pub project_id: ProjectId,
}
