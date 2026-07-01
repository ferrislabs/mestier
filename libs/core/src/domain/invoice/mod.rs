use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
	CustomerContextId, CustomerId, LegalMentionTemplateId, OrganizationContextId, OrganizationId,
	ServiceRateId, ServiceRateUnit,
};
use crate::domain::quote::QuoteId;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct InvoiceId(pub Uuid);

impl FromStr for InvoiceId {
	type Err = uuid::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Uuid::from_str(s).map(InvoiceId)
	}
}

impl Display for InvoiceId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct InvoiceLineId(pub Uuid);

impl FromStr for InvoiceLineId {
	type Err = uuid::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Uuid::from_str(s).map(InvoiceLineId)
	}
}

impl Display for InvoiceLineId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatus {
	Draft,
	Sent,
	PartiallyPaid,
	Paid,
	Cancelled,
}

impl InvoiceStatus {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Draft => "DRAFT",
			Self::Sent => "SENT",
			Self::PartiallyPaid => "PARTIALLY_PAID",
			Self::Paid => "PAID",
			Self::Cancelled => "CANCELLED",
		}
	}
}

impl Display for InvoiceStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl FromStr for InvoiceStatus {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"DRAFT" => Ok(Self::Draft),
			"SENT" => Ok(Self::Sent),
			"PARTIALLY_PAID" => Ok(Self::PartiallyPaid),
			"PAID" => Ok(Self::Paid),
			"CANCELLED" => Ok(Self::Cancelled),
			other => Err(format!("invalid invoice status `{other}`")),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceType {
	Standard,
	Deposit,
	Balance,
}

impl InvoiceType {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Standard => "STANDARD",
			Self::Deposit => "DEPOSIT",
			Self::Balance => "BALANCE",
		}
	}
}

impl Display for InvoiceType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl FromStr for InvoiceType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"STANDARD" => Ok(Self::Standard),
			"DEPOSIT" => Ok(Self::Deposit),
			"BALANCE" => Ok(Self::Balance),
			other => Err(format!("invalid invoice type `{other}`")),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvoiceLine {
	pub id: InvoiceLineId,
	pub org_id: OrganizationId,
	pub invoice_id: InvoiceId,
	pub service_rate_id: Option<ServiceRateId>,
	pub label: String,
	pub quantity: Decimal,
	pub unit: ServiceRateUnit,
	pub unit_price_cents: i32,
	pub vat_rate: Decimal,
	pub notes: Option<String>,
	pub photo_keys: Vec<String>,
	pub deleted_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Invoice {
	pub id: InvoiceId,
	pub org_id: OrganizationId,
	pub customer_id: CustomerId,
	pub customer_context_id: CustomerContextId,
	pub emitter_context_id: Option<OrganizationContextId>,
	pub reference: Option<String>,
	pub title: String,
	pub status: InvoiceStatus,
	pub invoice_type: InvoiceType,
	pub source_quote_id: Option<QuoteId>,
	pub parent_invoice_id: Option<InvoiceId>,
	pub deposit_basis: Option<String>,
	pub deposit_value: Option<Decimal>,
	pub deposit_amount_cents: Option<i32>,
	pub total_ht_cents: i32,
	pub total_vat_cents: i32,
	pub total_ttc_cents: i32,
	pub total_cents: i32,
	pub amount_paid_cents: i32,
	pub pdf_key: Option<String>,
	pub issued_at: Option<DateTime<Utc>>,
	pub due_at: Option<DateTime<Utc>>,
	pub lines: Vec<InvoiceLine>,
	pub legal_mention_template_ids: Vec<LegalMentionTemplateId>,
	pub deleted_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn invoice_status_parses_known_values() {
		assert_eq!("DRAFT".parse::<InvoiceStatus>().unwrap(), InvoiceStatus::Draft);
		assert_eq!("SENT".parse::<InvoiceStatus>().unwrap(), InvoiceStatus::Sent);
		assert_eq!(
			"PARTIALLY_PAID".parse::<InvoiceStatus>().unwrap(),
			InvoiceStatus::PartiallyPaid
		);
		assert_eq!("PAID".parse::<InvoiceStatus>().unwrap(), InvoiceStatus::Paid);
		assert_eq!(
			"CANCELLED".parse::<InvoiceStatus>().unwrap(),
			InvoiceStatus::Cancelled
		);
	}

	#[test]
	fn invoice_status_rejects_unknown_values() {
		assert!("APPROVED".parse::<InvoiceStatus>().is_err());
	}

	#[test]
	fn invoice_type_parses_known_values() {
		assert_eq!("STANDARD".parse::<InvoiceType>().unwrap(), InvoiceType::Standard);
		assert_eq!("DEPOSIT".parse::<InvoiceType>().unwrap(), InvoiceType::Deposit);
		assert_eq!("BALANCE".parse::<InvoiceType>().unwrap(), InvoiceType::Balance);
	}

	#[test]
	fn invoice_id_parses_uuid() {
		let uuid = Uuid::new_v4();
		let parsed = InvoiceId::from_str(&uuid.to_string()).unwrap();
		assert_eq!(parsed.0, uuid);
	}
}
