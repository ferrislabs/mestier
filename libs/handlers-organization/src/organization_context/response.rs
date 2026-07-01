use chrono::{DateTime, Utc};
use mestier_core::{OrganizationContext, OrganizationContextId, OrganizationId};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct OrganizationContextResponse {
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
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl From<OrganizationContext> for OrganizationContextResponse {
	fn from(value: OrganizationContext) -> Self {
		Self {
			id: value.id,
			org_id: value.org_id,
			label: value.label,
			address_line: value.address_line,
			postal_code: value.postal_code,
			city: value.city,
			country: value.country,
			siret: value.siret,
			rcs: value.rcs,
			ape: value.ape,
			vat_intracom: value.vat_intracom,
			iban: value.iban,
			bic: value.bic,
			created_at: value.created_at,
			updated_at: value.updated_at,
		}
	}
}
