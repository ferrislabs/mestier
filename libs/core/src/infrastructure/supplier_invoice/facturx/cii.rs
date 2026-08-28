//! Deserializes the Cross Industry Invoice (CII) XML extracted by
//! [`super::attachment`] into a [`ParsedSupplierInvoice`].
//!
//! The structs below are not a CII parser — CII (`UN/CEFACT
//! CrossIndustryInvoice`) is a multi-hundred-page schema with dozens of
//! trade parties, allowances, references and payment means this product
//! has no use for. They are a hand-written subset covering exactly the
//! fields `SupplierInvoice` needs: the seller's identity, the invoice
//! number and dates, the lines, and the stated totals. No crate on
//! crates.io generates or hand-maintains a CII binding (unlike, say,
//! UBL, which has several) — the schema's own complexity is presumably
//! why — so hand-writing this subset against `quick-xml`'s serde support
//! is the same "solved building blocks, not a solved product" trade-off
//! the module doc comment on `infrastructure::supplier_invoice::facturx`
//! makes for the PDF side.
//!
//! Two things about `quick-xml`'s serde mapping matter for every struct
//! below:
//! - it matches elements and attributes by **local name only** — CII's own
//!   `rsm:`/`ram:`/`udt:` namespace prefixes are stripped before matching,
//!   so `#[serde(rename = "ExchangedDocument")]` is correct and
//!   `#[serde(rename = "rsm:ExchangedDocument")]` silently never matches
//!   anything (this cost an early iteration of this module a `missing
//!   field` error against a real sample before the cause was traced to
//!   `quick_xml::de::key`'s doc comment);
//! - an XML attribute becomes a struct field renamed `@attr`, and an
//!   element's own text content becomes a field renamed `$text` — both
//!   used below wherever a CII element carries both (e.g.
//!   `<ram:ID schemeID="0002">123456782</ram:ID>`).

use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::Deserialize;

use crate::domain::supplier_invoice::ports::{
    ParsedSupplierInvoice, ParsedSupplierInvoiceLine, SupplierInvoiceParseError,
};

pub(super) fn parse(xml_bytes: &[u8]) -> Result<ParsedSupplierInvoice, SupplierInvoiceParseError> {
    let xml = std::str::from_utf8(xml_bytes)
        .map_err(|e| xml_error(format!("attachment is not valid UTF-8: {e}")))?;

    let document: CrossIndustryInvoice =
        quick_xml::de::from_str(xml).map_err(|e| xml_error(e.to_string()))?;

    convert(document)
}

#[derive(Debug, Deserialize)]
struct CrossIndustryInvoice {
    #[serde(rename = "ExchangedDocument")]
    exchanged_document: ExchangedDocument,
    #[serde(rename = "SupplyChainTradeTransaction")]
    trade_transaction: SupplyChainTradeTransaction,
}

#[derive(Debug, Deserialize)]
struct ExchangedDocument {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "IssueDateTime")]
    issue_date_time: DateTimeGroup,
}

/// The recurring `<ram:*DateTime><udt:DateTimeString format="...">value</udt:DateTimeString></ram:*DateTime>`
/// shape, reused by `IssueDateTime` and `DueDateDateTime` alike.
#[derive(Debug, Deserialize)]
struct DateTimeGroup {
    #[serde(rename = "DateTimeString")]
    date_time_string: FormattedDateTime,
}

#[derive(Debug, Deserialize)]
struct FormattedDateTime {
    #[serde(rename = "@format")]
    format: String,
    #[serde(rename = "$text")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct SupplyChainTradeTransaction {
    /// Repeats as a direct, contiguous sibling of the header elements
    /// below (the real CII schema orders every line item before the
    /// header groups) — no wrapping container, so `Vec<LineItem>` collects
    /// them without needing the `overlapped-lists` feature.
    #[serde(rename = "IncludedSupplyChainTradeLineItem", default)]
    line_items: Vec<LineItem>,
    #[serde(rename = "ApplicableHeaderTradeAgreement")]
    trade_agreement: TradeAgreement,
    #[serde(rename = "ApplicableHeaderTradeSettlement")]
    trade_settlement: TradeSettlement,
}

#[derive(Debug, Deserialize)]
struct TradeAgreement {
    #[serde(rename = "SellerTradeParty")]
    seller: SellerTradeParty,
}

#[derive(Debug, Deserialize)]
struct SellerTradeParty {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "SpecifiedLegalOrganization", default)]
    legal_organization: Option<LegalOrganization>,
    #[serde(rename = "SpecifiedTaxRegistration", default)]
    tax_registrations: Vec<TaxRegistration>,
}

#[derive(Debug, Deserialize)]
struct LegalOrganization {
    #[serde(rename = "ID", default)]
    id: Option<IdentifierWithScheme>,
}

#[derive(Debug, Deserialize)]
struct TaxRegistration {
    #[serde(rename = "ID")]
    id: IdentifierWithScheme,
}

/// An identifier element carrying its scheme as an attribute — e.g.
/// `<ram:ID schemeID="VA">FR11123456782</ram:ID>` for a VAT number, or
/// `schemeID="0002"` (SIREN) for a legal registration number. `scheme_id`
/// is read (to prefer a `VA`-scheme tax registration over any other) but
/// this product does not otherwise validate or interpret scheme codes.
#[derive(Debug, Deserialize)]
struct IdentifierWithScheme {
    #[serde(rename = "@schemeID", default)]
    scheme_id: Option<String>,
    #[serde(rename = "$text")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct TradeSettlement {
    #[serde(rename = "InvoiceCurrencyCode")]
    currency_code: String,
    #[serde(rename = "SpecifiedTradePaymentTerms", default)]
    payment_terms: Option<PaymentTerms>,
    #[serde(rename = "SpecifiedTradeSettlementHeaderMonetarySummation", default)]
    monetary_summation: Option<HeaderMonetarySummation>,
}

#[derive(Debug, Deserialize)]
struct PaymentTerms {
    #[serde(rename = "DueDateDateTime", default)]
    due_date_date_time: Option<DateTimeGroup>,
}

#[derive(Debug, Deserialize)]
struct HeaderMonetarySummation {
    #[serde(rename = "TaxBasisTotalAmount", default)]
    tax_basis_total_amount: Option<String>,
    #[serde(rename = "GrandTotalAmount", default)]
    grand_total_amount: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LineItem {
    #[serde(rename = "SpecifiedTradeProduct")]
    product: TradeProduct,
    #[serde(rename = "SpecifiedLineTradeAgreement")]
    agreement: LineTradeAgreement,
    #[serde(rename = "SpecifiedLineTradeDelivery")]
    delivery: LineTradeDelivery,
    #[serde(rename = "SpecifiedLineTradeSettlement")]
    settlement: LineTradeSettlement,
}

#[derive(Debug, Deserialize)]
struct TradeProduct {
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Debug, Deserialize)]
struct LineTradeAgreement {
    #[serde(rename = "NetPriceProductTradePrice")]
    net_price: NetPrice,
}

#[derive(Debug, Deserialize)]
struct NetPrice {
    #[serde(rename = "ChargeAmount")]
    charge_amount: String,
}

#[derive(Debug, Deserialize)]
struct LineTradeDelivery {
    #[serde(rename = "BilledQuantity")]
    billed_quantity: Quantity,
}

#[derive(Debug, Deserialize)]
struct Quantity {
    #[serde(rename = "@unitCode", default)]
    unit_code: Option<String>,
    #[serde(rename = "$text")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct LineTradeSettlement {
    #[serde(rename = "ApplicableTradeTax", default)]
    applicable_trade_tax: Option<LineTradeTax>,
    #[serde(rename = "SpecifiedTradeSettlementLineMonetarySummation")]
    monetary_summation: LineMonetarySummation,
}

#[derive(Debug, Deserialize)]
struct LineTradeTax {
    #[serde(rename = "RateApplicablePercent", default)]
    rate_applicable_percent: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LineMonetarySummation {
    #[serde(rename = "LineTotalAmount")]
    line_total_amount: String,
}

fn convert(
    document: CrossIndustryInvoice,
) -> Result<ParsedSupplierInvoice, SupplierInvoiceParseError> {
    let seller = document.trade_transaction.trade_agreement.seller;
    let settlement = document.trade_transaction.trade_settlement;

    let issued_on = parse_cii_date(&document.exchanged_document.issue_date_time.date_time_string)?;
    let due_on = settlement
        .payment_terms
        .as_ref()
        .and_then(|terms| terms.due_date_date_time.as_ref())
        .map(|due| parse_cii_date(&due.date_time_string))
        .transpose()?;

    let supplier_registration_number = seller
        .legal_organization
        .as_ref()
        .and_then(|org| org.id.as_ref())
        .map(|id| id.value.clone());

    let supplier_vat_number = seller
        .tax_registrations
        .iter()
        .find(|registration| registration.id.scheme_id.as_deref() == Some("VA"))
        .or_else(|| seller.tax_registrations.first())
        .map(|registration| registration.id.value.clone());

    let (stated_net_cents, stated_gross_cents) = match &settlement.monetary_summation {
        Some(summation) => (
            summation
                .tax_basis_total_amount
                .as_deref()
                .map(|value| scaled_i32(value, 100))
                .transpose()?,
            summation
                .grand_total_amount
                .as_deref()
                .map(|value| scaled_i32(value, 100))
                .transpose()?,
        ),
        None => (None, None),
    };

    let lines = document
        .trade_transaction
        .line_items
        .into_iter()
        .map(convert_line)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ParsedSupplierInvoice {
        supplier_name: seller.name,
        supplier_registration_number,
        supplier_vat_number,
        number: document.exchanged_document.id,
        issued_on,
        due_on,
        currency: settlement.currency_code,
        lines,
        stated_net_cents,
        stated_gross_cents,
    })
}

fn convert_line(line: LineItem) -> Result<ParsedSupplierInvoiceLine, SupplierInvoiceParseError> {
    let quantity = Decimal::from_str(line.delivery.billed_quantity.value.trim())
        .map_err(|e| xml_error(format!("invalid line quantity: {e}")))?;

    let vat_rate_basis_points = line
        .settlement
        .applicable_trade_tax
        .as_ref()
        .and_then(|tax| tax.rate_applicable_percent.as_deref())
        .map(|value| scaled_i32(value, 100))
        .transpose()?;

    Ok(ParsedSupplierInvoiceLine {
        label: line.product.name,
        quantity,
        unit: line.delivery.billed_quantity.unit_code,
        unit_price_cents: scaled_i32(&line.agreement.net_price.charge_amount, 100)?,
        line_total_cents: scaled_i32(&line.settlement.monetary_summation.line_total_amount, 100)?,
        vat_rate_basis_points,
    })
}

fn parse_cii_date(formatted: &FormattedDateTime) -> Result<NaiveDate, SupplierInvoiceParseError> {
    match formatted.format.as_str() {
        // "102" is the only date-only qualified format CII actually uses
        // for invoice-level dates (`qdt`/`udt` `FormattedDateTimeType`):
        // `CCYYMMDD`. Other codes exist in the wider standard (date+time,
        // week numbers, ...) but no field this product reads is specified
        // against them.
        "102" => NaiveDate::parse_from_str(&formatted.value, "%Y%m%d")
            .map_err(|e| xml_error(format!("invalid date `{}`: {e}", formatted.value))),
        other => Err(xml_error(format!(
            "unsupported date format code `{other}` (only `102`/CCYYMMDD is handled)"
        ))),
    }
}

/// Parses a decimal string and scales it to an integer (`* 100` for both a
/// currency amount in cents and a percentage in basis points — the same
/// arithmetic, just a different unit), rounding to the nearest whole
/// number the way `SupplierInvoiceLine`'s own cents fields always are.
fn scaled_i32(value: &str, scale: i64) -> Result<i32, SupplierInvoiceParseError> {
    let decimal = Decimal::from_str(value.trim())
        .map_err(|e| xml_error(format!("invalid number `{value}`: {e}")))?;

    (decimal * Decimal::from(scale))
        .round()
        .to_i64()
        .and_then(|scaled| i32::try_from(scaled).ok())
        .ok_or_else(|| xml_error(format!("number `{value}` is out of the supported range")))
}

fn xml_error(reason: String) -> SupplierInvoiceParseError {
    SupplierInvoiceParseError::XmlParsing(reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = include_str!("fixtures/sample-en16931.xml");

    #[test]
    fn parses_the_real_en16931_sample_into_the_fields_this_product_needs() {
        let parsed = parse(SAMPLE_XML.as_bytes()).unwrap();

        assert_eq!(parsed.number, "F20260023");
        assert_eq!(parsed.supplier_name, "LE FOURNISSEUR");
        assert_eq!(
            parsed.supplier_registration_number.as_deref(),
            Some("123456782")
        );
        assert_eq!(parsed.supplier_vat_number.as_deref(), Some("FR11123456782"));
        assert_eq!(
            parsed.issued_on,
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()
        );
        assert_eq!(parsed.due_on, NaiveDate::from_ymd_opt(2026, 3, 2));
        assert_eq!(parsed.currency, "EUR");
        assert_eq!(parsed.stated_net_cents, Some(10_000));
        assert_eq!(parsed.stated_gross_cents, Some(10_490));
        assert_eq!(parsed.lines.len(), 3);

        let first = &parsed.lines[0];
        assert_eq!(first.label, "PRESTATION SUPPORT");
        assert_eq!(first.unit_price_cents, 6_000);
        assert_eq!(first.line_total_cents, 6_000);
        assert_eq!(first.vat_rate_basis_points, Some(0));
        assert_eq!(first.unit.as_deref(), Some("C62"));
    }

    #[test]
    fn refuses_xml_missing_a_required_element_as_an_xml_parsing_error() {
        let broken =
            SAMPLE_XML.replace("<ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>", "");

        let error = parse(broken.as_bytes()).unwrap_err();

        assert!(matches!(error, SupplierInvoiceParseError::XmlParsing(_)));
    }

    #[test]
    fn refuses_bytes_that_are_not_xml_at_all() {
        let error = parse(b"not xml").unwrap_err();

        assert!(matches!(error, SupplierInvoiceParseError::XmlParsing(_)));
    }
}
