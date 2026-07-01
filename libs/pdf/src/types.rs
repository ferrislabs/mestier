use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerInfo {
	pub name: String,
	pub address: Option<String>,
	pub siret: Option<String>,
	pub rcs: Option<String>,
	pub ape: Option<String>,
	pub vat_intracom: Option<String>,
	pub iban: Option<String>,
	pub bic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerInfo {
	pub name: String,
	pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineSnapshot {
	pub label: String,
	pub quantity: String,
	pub unit: String,
	pub unit_price_cents: i64,
	pub vat_rate: String,
	pub line_ht_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatBucketSnapshot {
	pub rate: String,
	pub base_ht_cents: i64,
	pub vat_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSnapshot {
	pub kind: String,
	pub reference: Option<String>,
	pub title: String,
	pub issued_date: Option<String>,
	pub due_date: Option<String>,
	pub seller: SellerInfo,
	pub customer: CustomerInfo,
	pub lines: Vec<LineSnapshot>,
	pub vat_breakdown: Vec<VatBucketSnapshot>,
	pub total_ht_cents: i64,
	pub total_vat_cents: i64,
	pub total_ttc_cents: i64,
	pub deposit_label: Option<String>,
	pub deposit_amount_cents: Option<i64>,
	pub remaining_cents: Option<i64>,
	pub legal_mentions: Vec<String>,
	pub footer: Option<String>,
	pub payment_terms: Option<String>,
}

#[derive(Debug, Error)]
pub enum PdfError {
	#[error("Typst compilation error: {0}")]
	Compile(String),

	#[error("PDF export error: {0}")]
	Export(String),

	#[error("JSON serialization error: {0}")]
	Serialize(#[from] serde_json::Error),
}
