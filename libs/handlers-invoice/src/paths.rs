use axum_extra::routing::TypedPath;
use mestier_core::{InvoiceId, OrganizationId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/invoices")]
pub struct InvoicesPath {
	pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}")]
pub struct InvoicePath {
	pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/status")]
pub struct InvoiceStatusPath {
	pub invoice_id: InvoiceId,
}
