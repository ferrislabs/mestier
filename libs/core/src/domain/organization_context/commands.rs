use crate::{OrganizationContextId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateOrganizationContextCommand {
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
}

#[derive(Debug, Clone)]
pub struct UpdateOrganizationContextCommand {
	pub id: OrganizationContextId,
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
}
