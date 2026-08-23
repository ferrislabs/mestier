use authz::Subject;

use crate::{
    UserId,
    domain::organization::{OrganizationId, legal_identity::VatStatus},
};

#[derive(Debug, Clone)]
pub struct CreateOrganizationCommand {
    pub name: String,
    pub slug: String,
    pub owner_id: UserId,
}

#[derive(Debug, Clone)]
pub struct UpdateOrganizationCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    /// Whether the field app's home screen offers clocking in/out — see
    /// `Organization::field_clock_enabled`'s own doc. Not a `PATCH`-style
    /// double-Option: this command mirrors `name`/`slug` in always being a
    /// full replace, echoed by the caller on every save.
    pub field_clock_enabled: bool,
}

/// Replaces the whole legal-identity block in one call: the settings
/// section (#311) is a single form covering identity, address, VAT and
/// insurance, so a field left `None` here means "clear it", not "leave
/// unchanged" — the caller always resends the section in full.
#[derive(Debug, Clone)]
pub struct UpdateLegalIdentityCommand {
    pub actor: Subject,
    pub id: OrganizationId,
    pub legal_name: Option<String>,
    pub legal_form: Option<String>,
    pub registration_number: Option<String>,
    pub vat_status: Option<VatStatus>,
    pub share_capital_cents: Option<i64>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub insurance_mention: Option<String>,
}
