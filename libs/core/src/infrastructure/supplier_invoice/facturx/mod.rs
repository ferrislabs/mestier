//! #337: parses a Factur-X supplier invoice file into a
//! [`ParsedSupplierInvoice`], implementing the
//! [`SupplierInvoiceParser`] port two steps: [`attachment`] pulls the
//! embedded CII XML out of the PDF/A-3 container, [`cii`] deserializes it.
//!
//! ## What this is built on, and what was rejected
//!
//! Factur-X reception is "extract an attachment from a PDF/A-3, then parse
//! a CII XML profile" — both solved problems individually, with nothing on
//! crates.io solving the combination end to end:
//!
//! - **PDF/A-3 attachment extraction**: `lopdf` reads the PDF object model
//!   directly (`Document`, `Dictionary`, `Stream`), which is exactly what
//!   walking `Root/Names/EmbeddedFiles` down to the embedded stream needs.
//!   Considered and rejected: `pdf-extract` and similar — those extract
//!   *text* from a PDF's content streams, not named file attachments, a
//!   different problem entirely; and reimplementing PDF/A-3 parsing from
//!   the ISO 32000 spec by hand, which would be reinventing exactly what
//!   `lopdf` already does.
//! - **The CII XML profile**: no crate hand-maintains or generates a CII
//!   binding (UBL, the sibling e-invoicing format, has several bindings on
//!   crates.io — CII apparently does not). `quick-xml`'s `serialize`
//!   feature deserializes XML through `serde`, so [`cii`] only has to
//!   hand-write the subset of the schema this product reads (supplier
//!   identity, invoice number, dates, lines, VAT) rather than a parser for
//!   CII's full multi-hundred-page specification (which also covers order,
//!   despatch and remittance messages this product never receives).
//!   Considered and rejected: `serde-xml-rs`, which does not distinguish
//!   attributes from elements at all, and both `IdentifierWithScheme`
//!   (`<ram:ID schemeID="VA">...</ram:ID>`) and `Quantity`
//!   (`<ram:BilledQuantity unitCode="C62">...</ram:BilledQuantity>`) in
//!   `cii` need that distinction to read the scheme/unit code.

mod attachment;
mod cii;

use crate::domain::supplier_invoice::ports::{
    ParsedSupplierInvoice, SupplierInvoiceParseError, SupplierInvoiceParser,
};

/// Stateless: holds no configuration and opens no connection, so one
/// instance is reused across every import rather than built per call.
#[derive(Debug, Default, Clone, Copy)]
pub struct FacturXParser;

impl SupplierInvoiceParser for FacturXParser {
    fn parse(&self, bytes: &[u8]) -> Result<ParsedSupplierInvoice, SupplierInvoiceParseError> {
        let xml = attachment::extract_embedded_xml(bytes)?;
        cii::parse(&xml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_PDF: &[u8] = include_bytes!("fixtures/valid.pdf");
    const MALFORMED_PDF: &[u8] = include_bytes!("fixtures/malformed.pdf");

    #[test]
    fn parses_a_real_factur_x_pdf_end_to_end() {
        let parsed = FacturXParser
            .parse(VALID_PDF)
            .expect("a valid Factur-X PDF");

        assert_eq!(parsed.number, "F20260023");
        assert_eq!(parsed.supplier_name, "LE FOURNISSEUR");
        assert_eq!(parsed.lines.len(), 3);
    }

    #[test]
    fn surfaces_a_truncated_pdf_as_an_attachment_extraction_failure() {
        let error = FacturXParser.parse(MALFORMED_PDF).unwrap_err();

        assert!(matches!(
            error,
            SupplierInvoiceParseError::AttachmentExtraction(_)
        ));
    }
}
