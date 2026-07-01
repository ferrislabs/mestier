use axum_extra::routing::TypedPath;
use mestier_core::{InvoiceId, OrganizationId, QuoteId};
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

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/quotes/{quote_id}/convert-to-invoice")]
pub struct QuoteConvertPath {
	pub quote_id: QuoteId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/deposit-invoice")]
pub struct InvoiceDepositPath {
	pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/balance-invoice")]
pub struct InvoiceBalancePath {
	pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/issue")]
pub struct InvoiceIssuePath {
	pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/pdf")]
pub struct InvoicePdfPath {
	pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invoices/{invoice_id}/share")]
pub struct InvoiceSharePath {
	pub invoice_id: InvoiceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/public/documents/{token}")]
pub struct PublicDocumentPath {
	pub token: String,
}
