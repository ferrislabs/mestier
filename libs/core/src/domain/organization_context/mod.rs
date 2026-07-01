use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct OrganizationContextId(pub Uuid);

impl FromStr for OrganizationContextId {
	type Err = uuid::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Uuid::from_str(s).map(OrganizationContextId)
	}
}

impl Display for OrganizationContextId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrganizationContext {
	pub id: OrganizationContextId,
	pub org_id: OrganizationId,
	pub label: String,
	pub address_line: Option<String>,
	pub postal_code: Option<String>,
	pub city: Option<String>,
	pub country: Option<String>,
	pub siret: Option<String>,
	pub rcs: Option<String>,
	pub ape: Option<String>,
	pub vat_intracom: Option<String>,
	pub iban: Option<String>,
	pub bic: Option<String>,
	pub deleted_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn organization_context_id_parses_uuid() {
		let uuid = Uuid::new_v4();
		let parsed = OrganizationContextId::from_str(&uuid.to_string()).unwrap();

		assert_eq!(parsed.0, uuid);
	}

	#[test]
	fn organization_context_id_rejects_invalid_uuid() {
		assert!(OrganizationContextId::from_str("not-a-uuid").is_err());
	}
}
