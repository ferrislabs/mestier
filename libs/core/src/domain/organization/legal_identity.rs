use crate::domain::{customer::Customer, organization::Organization};

/// Whether an organization charges VAT on what it issues, and — either way —
/// the fact that makes the document valid without it: the number when it
/// does, the legal basis for the exemption when it does not. A nullable VAT
/// number would make "under the franchise" indistinguishable from "nobody
/// filled this in yet", and that ambiguity is exactly what a document must
/// never carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VatStatus {
    Subject { vat_number: String },
    NotSubject { basis: String },
}

/// A postal address, complete enough to appear on a legal document.
/// `line2` stays optional even here — a second address line is genuinely
/// optional information, unlike the rest of the address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationAddress {
    pub line1: String,
    pub line2: Option<String>,
    pub postal_code: String,
    pub city: String,
    pub country: String,
}

/// The name of a field missing from an organization's legal identity, in the
/// vocabulary the API and the settings form both use: stable across
/// releases, because a refusal message (#314) and a "jump to this field"
/// link (#311) are both built from it.
pub type MissingLegalIdentityField = &'static str;

/// An organization's identity, complete enough that a document can be
/// legally issued in its name. Built only through `try_from_organization`:
/// there is no way to construct one with a field missing, which turns "this
/// quote cannot be sent, the SIRET is missing" into a compile-time shape
/// rather than a runtime check somebody forgets in a second code path
/// (today's PDF export, tomorrow's invoice).
///
/// The field list is a starting point pending confirmation against a
/// current source, per #284 — not French legal advice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegalIdentity {
    pub legal_name: String,
    pub legal_form: String,
    pub registration_number: String,
    pub vat_status: VatStatus,
    /// Only meaningful for legal forms that have one (SARL, SAS...): a sole
    /// trader legitimately has none, so this never gates completeness.
    pub share_capital_cents: Option<i64>,
    pub address: OrganizationAddress,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub insurance_mention: String,
}

impl LegalIdentity {
    /// `Err` lists every missing field, not just the first: the caller — a
    /// refused PDF export, a settings page banner — needs the whole picture
    /// in one round trip rather than fixing one field at a time to discover
    /// the next.
    pub fn try_from_organization(
        organization: &Organization,
    ) -> Result<Self, Vec<MissingLegalIdentityField>> {
        let mut missing = Vec::new();

        let legal_name = require(&organization.legal_name, "legal_name", &mut missing);
        let legal_form = require(&organization.legal_form, "legal_form", &mut missing);
        let registration_number = require(
            &organization.registration_number,
            "registration_number",
            &mut missing,
        );
        let insurance_mention = require(
            &organization.insurance_mention,
            "insurance_mention",
            &mut missing,
        );
        if organization.vat_status.is_none() {
            missing.push("vat_status");
        }
        let line1 = require(&organization.address_line1, "address_line1", &mut missing);
        let postal_code = require(
            &organization.address_postal_code,
            "address_postal_code",
            &mut missing,
        );
        let city = require(&organization.address_city, "address_city", &mut missing);
        let country = require(
            &organization.address_country,
            "address_country",
            &mut missing,
        );

        if !missing.is_empty() {
            return Err(missing);
        }

        Ok(LegalIdentity {
            // Every `.expect` below is checked by the `missing.is_empty()`
            // guard above: reaching this arm means `require` returned
            // `Some` for each of these fields.
            legal_name: legal_name.expect("checked above"),
            legal_form: legal_form.expect("checked above"),
            registration_number: registration_number.expect("checked above"),
            vat_status: organization.vat_status.clone().expect("checked above"),
            share_capital_cents: organization.share_capital_cents,
            address: OrganizationAddress {
                line1: line1.expect("checked above"),
                line2: organization.address_line2.clone(),
                postal_code: postal_code.expect("checked above"),
                city: city.expect("checked above"),
                country: country.expect("checked above"),
            },
            contact_email: organization.contact_email.clone(),
            contact_phone: organization.contact_phone.clone(),
            insurance_mention: insurance_mention.expect("checked above"),
        })
    }
}

/// `Some(trimmed-ish value)` when present and non-blank, else records `name`
/// as missing and returns `None`. Blank strings are treated the same as
/// absent: the database only guarantees non-blank when the field is set at
/// all (see the `chk_organizations_*_not_blank` constraints), but the
/// pattern is not enforced everywhere a value could theoretically arrive
/// blank once whitespace-only input reaches this far.
fn require(
    field: &Option<String>,
    name: MissingLegalIdentityField,
    missing: &mut Vec<MissingLegalIdentityField>,
) -> Option<String> {
    match field {
        Some(value) if !value.trim().is_empty() => Some(value.clone()),
        _ => {
            missing.push(name);
            None
        }
    }
}

/// The name of a fact missing before an invoice can actually be
/// transmitted electronically — `MissingLegalIdentityField`'s counterpart
/// for the facts #341 adds on top of `LegalIdentity`.
pub type MissingElectronicInvoicingFact = &'static str;

/// The facts an electronically-transmitted invoice needs beyond
/// `LegalIdentity`: a complete issuer identity, and the customer's own
/// SIREN. Built only through [`Self::try_new`] — same device as
/// `LegalIdentity::try_from_organization` — so "this invoice cannot be
/// transmitted, the customer has no SIREN" is a type-level fact rather
/// than a check a PDF export or a PDP transmission path can separately
/// forget.
///
/// Whether the customer's SIREN is actually *required* depends on the
/// customer itself being VAT-subject, per the reform text verified in
/// #341's migration comment — this codebase does not yet track a
/// customer's own VAT status, so today this always requires the SIREN
/// when present on the invoice's counterparty. Narrowing that to "only
/// when the customer is VAT-subject" is an explicit follow-up, not
/// something this type gets right yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronicInvoicingFacts {
    pub issuer: LegalIdentity,
    pub customer_registration_number: String,
}

impl ElectronicInvoicingFacts {
    pub fn try_new(
        organization: &Organization,
        customer: &Customer,
    ) -> Result<Self, Vec<MissingElectronicInvoicingFact>> {
        let mut missing: Vec<MissingElectronicInvoicingFact> = Vec::new();

        let issuer = match LegalIdentity::try_from_organization(organization) {
            Ok(issuer) => Some(issuer),
            Err(fields) => {
                missing.extend(fields);
                None
            }
        };

        let customer_registration_number = match &customer.registration_number {
            Some(value) if !value.trim().is_empty() => Some(value.clone()),
            _ => {
                missing.push("customer_registration_number");
                None
            }
        };

        if !missing.is_empty() {
            return Err(missing);
        }

        Ok(Self {
            issuer: issuer.expect("checked above"),
            customer_registration_number: customer_registration_number.expect("checked above"),
        })
    }

    /// As [`Self::try_new`], but from an issuer identity already frozen —
    /// an issued invoice's own `issuer_identity` — rather than a fresh
    /// `Organization` lookup.
    ///
    /// This is the constructor #342's `DocumentFormat` port is built
    /// against: generating an electronic document for an invoice already
    /// sent must read the facts as they were the instant it was issued, not
    /// the organization's current row, which may have changed since (a
    /// corrected address, a VAT status update) in a way that would make the
    /// regenerated file disagree with the one the customer already holds.
    /// `try_new` stays the right constructor everywhere a *live* lookup is
    /// actually wanted — a draft's own PDF preview, the settings page's own
    /// completeness banner — this one is not a replacement for it.
    pub fn from_frozen_issuer(
        issuer: LegalIdentity,
        customer: &Customer,
    ) -> Result<Self, Vec<MissingElectronicInvoicingFact>> {
        match &customer.registration_number {
            Some(value) if !value.trim().is_empty() => Ok(Self {
                issuer,
                customer_registration_number: value.clone(),
            }),
            _ => Err(vec!["customer_registration_number"]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UserId;
    use chrono::Utc;
    use uuid::Uuid;

    fn blank_organization() -> Organization {
        let now = Utc::now();
        Organization {
            id: crate::OrganizationId(Uuid::new_v4()),
            name: "Acme".into(),
            slug: "acme".into(),
            owner_id: UserId(Uuid::new_v4()),
            legal_name: None,
            legal_form: None,
            registration_number: None,
            vat_status: None,
            share_capital_cents: None,
            address_line1: None,
            address_line2: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            contact_email: None,
            contact_phone: None,
            insurance_mention: None,
            quote_number_prefix: "DEV".to_owned(),
            invoice_number_prefix: "FAC".to_owned(),
            field_clock_enabled: false,
            vat_on_debits: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn complete_organization() -> Organization {
        Organization {
            legal_name: Some("Acme SARL".into()),
            legal_form: Some("SARL".into()),
            registration_number: Some("123 456 789 00012".into()),
            vat_status: Some(VatStatus::Subject {
                vat_number: "FR12345678901".into(),
            }),
            share_capital_cents: Some(1_000_000),
            address_line1: Some("12 rue des Artisans".into()),
            address_line2: None,
            address_postal_code: Some("75001".into()),
            address_city: Some("Paris".into()),
            address_country: Some("FR".into()),
            contact_email: Some("contact@acme.fr".into()),
            contact_phone: None,
            insurance_mention: Some("RC Pro n°123456 - MAAF Assurances".into()),
            ..blank_organization()
        }
    }

    #[test]
    fn builds_from_a_complete_organization() {
        let identity = LegalIdentity::try_from_organization(&complete_organization()).unwrap();

        assert_eq!(identity.legal_name, "Acme SARL");
        assert_eq!(
            identity.vat_status,
            VatStatus::Subject {
                vat_number: "FR12345678901".into()
            }
        );
        assert_eq!(identity.address.city, "Paris");
        assert_eq!(identity.address.line2, None);
    }

    #[test]
    fn a_not_subject_organization_carries_the_exemption_basis() {
        let organization = Organization {
            vat_status: Some(VatStatus::NotSubject {
                basis: "Article 293 B du CGI".into(),
            }),
            ..complete_organization()
        };

        let identity = LegalIdentity::try_from_organization(&organization).unwrap();

        assert_eq!(
            identity.vat_status,
            VatStatus::NotSubject {
                basis: "Article 293 B du CGI".into()
            }
        );
    }

    #[test]
    fn a_blank_organization_reports_every_missing_field() {
        let err = LegalIdentity::try_from_organization(&blank_organization()).unwrap_err();

        assert_eq!(
            err,
            vec![
                "legal_name",
                "legal_form",
                "registration_number",
                "insurance_mention",
                "vat_status",
                "address_line1",
                "address_postal_code",
                "address_city",
                "address_country",
            ]
        );
    }

    #[test]
    fn reports_only_the_fields_actually_missing() {
        let organization = Organization {
            registration_number: None,
            ..complete_organization()
        };

        let err = LegalIdentity::try_from_organization(&organization).unwrap_err();

        assert_eq!(err, vec!["registration_number"]);
    }

    #[test]
    fn whitespace_only_values_count_as_missing() {
        let organization = Organization {
            legal_name: Some("   ".into()),
            ..complete_organization()
        };

        let err = LegalIdentity::try_from_organization(&organization).unwrap_err();

        assert_eq!(err, vec!["legal_name"]);
    }

    #[test]
    fn share_capital_absence_never_blocks_completeness() {
        let organization = Organization {
            share_capital_cents: None,
            ..complete_organization()
        };

        let identity = LegalIdentity::try_from_organization(&organization).unwrap();

        assert_eq!(identity.share_capital_cents, None);
    }

    fn customer_with_registration_number(value: Option<&str>) -> Customer {
        let now = Utc::now();
        Customer {
            id: crate::CustomerId(Uuid::new_v4()),
            organization_id: crate::OrganizationId(Uuid::new_v4()),
            status: crate::CustomerStatus::Client,
            pipeline_stage: crate::CustomerPipelineStage::Won,
            name: "Jean Dupont".into(),
            registration_number: value.map(str::to_owned),
            phone: None,
            email: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn electronic_invoicing_facts_build_from_a_complete_pair() {
        let facts = ElectronicInvoicingFacts::try_new(
            &complete_organization(),
            &customer_with_registration_number(Some("123 456 789")),
        )
        .unwrap();

        assert_eq!(facts.issuer.legal_name, "Acme SARL");
        assert_eq!(facts.customer_registration_number, "123 456 789");
    }

    #[test]
    fn electronic_invoicing_facts_report_a_missing_customer_siren() {
        let err = ElectronicInvoicingFacts::try_new(
            &complete_organization(),
            &customer_with_registration_number(None),
        )
        .unwrap_err();

        assert_eq!(err, vec!["customer_registration_number"]);
    }

    #[test]
    fn electronic_invoicing_facts_report_both_missing_sides_at_once() {
        let err = ElectronicInvoicingFacts::try_new(
            &blank_organization(),
            &customer_with_registration_number(None),
        )
        .unwrap_err();

        assert!(err.contains(&"legal_name"));
        assert!(err.contains(&"customer_registration_number"));
    }

    #[test]
    fn from_frozen_issuer_builds_from_an_already_complete_identity() {
        let issuer = LegalIdentity::try_from_organization(&complete_organization()).unwrap();

        let facts = ElectronicInvoicingFacts::from_frozen_issuer(
            issuer,
            &customer_with_registration_number(Some("123 456 789")),
        )
        .unwrap();

        assert_eq!(facts.issuer.legal_name, "Acme SARL");
        assert_eq!(facts.customer_registration_number, "123 456 789");
    }

    #[test]
    fn from_frozen_issuer_reports_a_missing_customer_siren() {
        let issuer = LegalIdentity::try_from_organization(&complete_organization()).unwrap();

        let err = ElectronicInvoicingFacts::from_frozen_issuer(
            issuer,
            &customer_with_registration_number(None),
        )
        .unwrap_err();

        assert_eq!(err, vec!["customer_registration_number"]);
    }
}
