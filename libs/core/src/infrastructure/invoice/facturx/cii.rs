//! Maps a Mestier [`Invoice`] into the [`en16931::Invoice`] semantic model,
//! then hands it to `en16931-formats`' CII writer — which validates against
//! the EN 16931 core profile *and* serialises in the same call
//! ([`en16931_formats::cii::to_string_for`]), refusing rather than handing
//! back XML for an invoice that does not satisfy it.
//!
//! ## Why this crate, and not a hand-written CII binding
//!
//! See `infrastructure::invoice::facturx`'s own module doc comment for the
//! full evaluation. In short: `en16931` is a semantic model whose 227 EN
//! 16931 core rules are tested against CEN's own conformance corpus and
//! KoSIT's published Schematron (223/223 CEN syntax-independent rules
//! covered, per its README), and `en16931-formats::cii` is the writer over
//! that model — this module only has to *map*, not implement, either the
//! business rules or the CII schema.
//!
//! ## What this mapping cannot make valid today, and why that is not a bug
//! here
//!
//! [`Customer`] carries no postal address at all — nothing in this codebase
//! has needed one before #342. EN 16931's `BG-8`/`BR-10`/`BR-11` require the
//! buyer's postal address, country included, unconditionally. So
//! [`to_en16931_invoice`] maps the buyer's address as empty (there is
//! nothing else to put there), and every invoice this mapping produces
//! **will be refused** by [`super::write_cii_xml`]'s call into the reference
//! validator, with a report naming exactly that gap — not a crash, not a
//! silently wrong document, a clean refusal with the missing business terms
//! named. Adding a postal address to `Customer` is the concrete next step
//! before Factur-X generation can succeed against a real customer; see this
//! issue's own PR description for the fuller note.
//!
//! `InvoiceLine` carries no unit-of-measure code either (`BT-130`,
//! mandatory, `BR-23`) — unlike `SupplierInvoiceLine`, which reads one off
//! an inbound Factur-X document. Every line here is mapped to unit code
//! `C62` (UN/CEFACT "one", the dimensionless count unit), which is what a
//! system that never tracked units defaults to; it is a real, valid EN
//! 16931 unit code, not a placeholder that happens to parse, but it is a
//! simplification worth a human's attention if a future invoice genuinely
//! bills in litres or metres.

use en16931::invoice::{
    Code, DocumentTotals, Invoice as En16931Invoice, InvoiceLine as En16931Line, Item, LineVat,
    Party, PostalAddress, PriceDetails, VatBreakdown,
};
use en16931::{
    Date, DocumentKind, Identifier, InvoiceAmount, Percentage, Quantity, UnitPriceAmount,
};
use rust_decimal::Decimal;

use crate::domain::invoice::ports::DocumentFormatError;
use crate::domain::organization::legal_identity::{
    ElectronicInvoicingFacts, LegalIdentity, VatStatus,
};
use crate::{Customer, Invoice, InvoiceKind, InvoiceLine};

/// Validates `invoice` against the EN 16931 core profile and serialises it
/// as CII, in one call — [`en16931_formats::cii::to_string_for`] does both,
/// refusing to hand back XML for a model that does not satisfy the profile
/// rather than serialising something no counterparty would accept. This is
/// the "reference validator" step this issue's own text asks for: not
/// veraPDF, not Chorus Pro (neither reachable from here), but
/// `en16931`'s rule engine, checked against CEN's own conformance corpus
/// and KoSIT's published Schematron — see this module's parent's own doc
/// comment for the fuller evaluation.
pub(super) fn write_cii_xml(invoice: &En16931Invoice) -> Result<String, DocumentFormatError> {
    en16931_formats::cii::to_string_for(invoice, &en16931::profiles::EN16931).map_err(|e| {
        DocumentFormatError::NotValid {
            profile: e.profile(),
            report: e.report().to_string(),
        }
    })
}

/// The scheme ISO 6523 assigns to France's SIRENE register — the same
/// scheme `infrastructure::supplier_invoice::facturx::cii` reads off an
/// inbound document's `SpecifiedLegalOrganization/ID/@schemeID`. Used here
/// both for the legal registration identifier and, absent anything else on
/// either party, for the electronic address BR-62/BR-63 require: a SIREN is
/// not really an EAS endpoint, but this codebase transmits Factur-X files
/// directly rather than over the Peppol network, so there is no real
/// endpoint to state, and a domestic receiver keys off the SIREN regardless.
const FR_SIRENE_SCHEME: &str = "0002";

/// UN/CEFACT Recommendation 20's "one" — the unit a line carries no more
/// specific unit code for. See this module's own doc comment.
const DEFAULT_UNIT_CODE: &str = "C62";

/// Infallible by construction: every field either party mapper below writes
/// is a value the semantic model already knows how to hold, per
/// `en16931-formats`' own "serialisation cannot fail" argument (`cii::mod`'s
/// doc comment) applied one step earlier, at the model itself. Whether the
/// result is *valid* is a separate, later question — see
/// [`super::write_cii_xml`].
pub(super) fn to_en16931_invoice(
    invoice: &Invoice,
    facts: &ElectronicInvoicingFacts,
    customer: &Customer,
) -> En16931Invoice {
    let type_code = match invoice.kind {
        InvoiceKind::CreditNote => "381",
        _ => "380",
    };
    let kind = match invoice.kind {
        InvoiceKind::CreditNote => DocumentKind::CreditNote,
        _ => DocumentKind::Invoice,
    };
    let issue_date = invoice
        .issued_at
        .map(to_en16931_date)
        .unwrap_or_else(|| Date::new(1970, 1, 1).expect("epoch is a calendar day"));

    // `en16931::invoice::Invoice` is `#[non_exhaustive]`: no struct-literal
    // construction from outside its own crate, `..Default::default()`
    // included, so this builds through `Invoice::builder` (the five terms
    // BR-01..BR-05 require) and sets everything else as plain field
    // assignment on the value it returns — allowed, since non-exhaustive
    // only forbids literal construction, not mutating a value already
    // owned.
    let mut inv = En16931Invoice::builder(
        "urn:cen.eu:en16931:2017", // overwritten by `to_string_for`'s own stamp; named for readability here
        invoice.number.clone().unwrap_or_default(),
        issue_date,
        type_code,
        "EUR", // the only currency this codebase's amounts are ever denominated in
    )
    .build();
    inv.kind = kind;
    inv.due_date = invoice.due_at.map(to_en16931_date);
    inv.seller = seller_party(facts);
    inv.buyer = buyer_party(customer, &facts.customer_registration_number);

    let (lines, vat_breakdown) = lines_and_vat_breakdown(invoice, facts);
    inv.lines = lines;
    inv.vat_breakdown = vat_breakdown;
    inv.totals = totals(invoice);

    inv
}

fn to_en16931_date(at: chrono::DateTime<chrono::Utc>) -> Date {
    use chrono::Datelike;
    Date::new(at.year(), at.month() as u8, at.day() as u8)
        .unwrap_or_else(|_| Date::new(1970, 1, 1).expect("epoch is a calendar day"))
}

fn seller_party(facts: &ElectronicInvoicingFacts) -> Party {
    let issuer = &facts.issuer;
    let registration = Identifier::schemed(issuer.registration_number.clone(), FR_SIRENE_SCHEME);

    Party {
        name: Some(issuer.legal_name.clone()),
        legal_registration: Some(registration),
        vat_identifier: match &issuer.vat_status {
            VatStatus::Subject { vat_number } => Some(vat_number.clone()),
            VatStatus::NotSubject { .. } => None,
        },
        additional_legal_information: Some(additional_legal_information(issuer)),
        electronic_address: Identifier::eas(issuer.registration_number.clone(), FR_SIRENE_SCHEME)
            .ok(),
        address: PostalAddress {
            line1: Some(issuer.address.line1.clone()),
            line2: issuer.address.line2.clone(),
            city: Some(issuer.address.city.clone()),
            post_code: Some(issuer.address.postal_code.clone()),
            country: Some(Code::new(issuer.address.country.clone())),
            ..PostalAddress::default()
        },
        ..Party::default()
    }
}

/// Free text carrying what `LegalIdentity` has no dedicated CII business
/// term for: the legal form, the insurance mention every French artisan
/// invoice must carry, and share capital when the legal form has one — the
/// same three facts `handlers_invoice::invoice::export_pdf`'s renderer
/// already prints, reused here as prose rather than re-derived.
fn additional_legal_information(issuer: &LegalIdentity) -> String {
    let mut parts = vec![issuer.legal_form.clone()];
    if let Some(capital_cents) = issuer.share_capital_cents {
        parts.push(format!(
            "Capital social {}.{:02} EUR",
            capital_cents / 100,
            capital_cents.unsigned_abs() % 100
        ));
    }
    parts.push(issuer.insurance_mention.clone());
    parts.join(" - ")
}

/// The buyer carries no address at all — see this module's own doc comment
/// on why, and on the refusal that leaves for [`super::write_cii_xml`] to
/// surface.
fn buyer_party(customer: &Customer, registration_number: &str) -> Party {
    Party {
        name: Some(customer.name.clone()),
        legal_registration: Some(Identifier::schemed(
            registration_number.to_owned(),
            FR_SIRENE_SCHEME,
        )),
        electronic_address: Identifier::eas(registration_number.to_owned(), FR_SIRENE_SCHEME).ok(),
        ..Party::default()
    }
}

/// One VAT category per distinct rate on the invoice's own lines, in the
/// vocabulary EN 16931's `VatCategory` code list (`UNCL 5305`) uses: `S`
/// (standard rated) for a positive rate, `Z` (zero rated) for `0`, `E`
/// (exempt) whenever the organization itself is not VAT-subject at all —
/// which is also the one case a line's own `vat_rate_basis_points` is
/// ignored, since the exemption is about the seller, not the line.
///
/// The taxable amount per group is **derived from the lines**, not
/// invented: `InvoiceVatBreakdownLine` (the domain's own persisted
/// breakdown) carries `vat_cents` per rate but not the taxable base that
/// produced it, so this groups the lines by rate and sums each line's own
/// net amount — the exact formula
/// `handlers_invoice::invoice::export_pdf::line_net_cents` already uses for
/// display, not a new computation. `vat_cents` itself is never recomputed:
/// each group's tax amount comes from the persisted
/// `InvoiceVatBreakdownLine` verbatim.
fn lines_and_vat_breakdown(
    invoice: &Invoice,
    facts: &ElectronicInvoicingFacts,
) -> (Vec<En16931Line>, Vec<VatBreakdown>) {
    let exempt = matches!(facts.issuer.vat_status, VatStatus::NotSubject { .. });

    let lines: Vec<En16931Line> = invoice
        .lines
        .iter()
        .enumerate()
        .map(|(index, line)| to_en16931_line(index, line, exempt))
        .collect();

    if exempt {
        let VatStatus::NotSubject { basis } = &facts.issuer.vat_status else {
            unreachable!("`exempt` is only true for `NotSubject`");
        };
        let taxable: i64 = invoice
            .lines
            .iter()
            .map(|l| i64::from(line_net_cents(l)))
            .sum();
        return (
            lines,
            vec![VatBreakdown {
                taxable_amount: InvoiceAmount::from_minor_units(taxable),
                tax_amount: InvoiceAmount::from_minor_units(0),
                category: Code::new("E"),
                rate: Some(Percentage::ZERO),
                exemption_reason: Some(basis.clone()),
                exemption_reason_code: None,
            }],
        );
    }

    let breakdown = invoice
        .vat_breakdown
        .iter()
        .map(|group| {
            let taxable: i64 = invoice
                .lines
                .iter()
                .filter(|l| l.vat_rate_basis_points.unwrap_or(0) == group.rate_bp)
                .map(|l| i64::from(line_net_cents(l)))
                .sum();
            let category = if group.rate_bp > 0 { "S" } else { "Z" };

            VatBreakdown {
                taxable_amount: InvoiceAmount::from_minor_units(taxable),
                tax_amount: InvoiceAmount::from_minor_units(i64::from(group.vat_cents)),
                category: Code::new(category),
                rate: Some(Percentage::new(Decimal::new(i64::from(group.rate_bp), 2))),
                exemption_reason: None,
                exemption_reason_code: None,
            }
        })
        .collect();

    (lines, breakdown)
}

fn to_en16931_line(index: usize, line: &InvoiceLine, exempt: bool) -> En16931Line {
    let category = if exempt {
        "E"
    } else if line.vat_rate_basis_points.unwrap_or(0) > 0 {
        "S"
    } else {
        "Z"
    };
    let rate = if exempt {
        Some(Percentage::ZERO)
    } else {
        Some(Percentage::new(Decimal::new(
            i64::from(line.vat_rate_basis_points.unwrap_or(0)),
            2,
        )))
    };

    En16931Line {
        id: (index + 1).to_string(),
        quantity: Quantity::new(line.quantity),
        unit_code: Code::new(DEFAULT_UNIT_CODE),
        net_amount: InvoiceAmount::from_minor_units(i64::from(line_net_cents(line))),
        price: PriceDetails {
            net_price: UnitPriceAmount::new(Decimal::new(i64::from(line.unit_price_cents), 2)),
            ..PriceDetails::default()
        },
        vat: LineVat {
            category: Code::new(category),
            rate,
        },
        item: Item {
            name: Some(line.label.clone()),
            ..Item::default()
        },
        note: None,
        order_line_reference: None,
        accounting_reference: None,
        object_identifier: None,
        period: None,
        allowances: vec![],
        charges: vec![],
    }
}

/// `quantity * unit_price_cents`, rounded to the nearest cent — the same
/// formula and the same rounding as
/// `handlers_invoice::invoice::export_pdf::line_net_cents`. Duplicated
/// rather than shared across a crate boundary that does not otherwise
/// exist between `handlers-invoice` and `mestier-core`; kept identical on
/// purpose; see this module's own doc comment on why it must never diverge.
fn line_net_cents(line: &InvoiceLine) -> i32 {
    use rust_decimal::prelude::ToPrimitive;

    (line.quantity * Decimal::from(line.unit_price_cents))
        .round_dp(0)
        .to_i32()
        .unwrap_or(0)
}

fn totals(invoice: &Invoice) -> DocumentTotals {
    let vat_total: i64 = invoice
        .vat_breakdown
        .iter()
        .map(|g| i64::from(g.vat_cents))
        .sum();

    DocumentTotals {
        line_total: InvoiceAmount::from_minor_units(i64::from(invoice.net_cents)),
        taxable_total: InvoiceAmount::from_minor_units(i64::from(invoice.net_cents)),
        vat_total: Some(InvoiceAmount::from_minor_units(vat_total)),
        gross_total: InvoiceAmount::from_minor_units(i64::from(invoice.gross_cents)),
        due: InvoiceAmount::from_minor_units(i64::from(invoice.gross_cents)),
        ..DocumentTotals::default()
    }
}
