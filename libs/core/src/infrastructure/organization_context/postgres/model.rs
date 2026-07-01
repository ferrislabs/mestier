use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{OrganizationContext, OrganizationContextId, OrganizationId};

#[derive(Debug, Clone)]
pub struct OrganizationContextRow {
	pub id: Uuid,
	pub org_id: Uuid,
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

impl From<OrganizationContextRow> for OrganizationContext {
	fn from(row: OrganizationContextRow) -> Self {
		Self {
			id: OrganizationContextId(row.id),
			org_id: OrganizationId(row.org_id),
			label: row.label,
			address_line: row.address_line,
			postal_code: row.postal_code,
			city: row.city,
			country: row.country,
			siret: row.siret,
			rcs: row.rcs,
			ape: row.ape,
			vat_intracom: row.vat_intracom,
			iban: row.iban,
			bic: row.bic,
			deleted_at: row.deleted_at,
			created_at: row.created_at,
			updated_at: row.updated_at,
		}
	}
}
