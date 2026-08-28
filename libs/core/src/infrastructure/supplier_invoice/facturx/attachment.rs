//! Pulls the embedded CII XML out of a Factur-X PDF/A-3 container.
//!
//! `lopdf` gives raw access to the PDF object graph (`Document`,
//! `Dictionary`, `Stream`) rather than a purpose-built "get me the
//! attachment" call — nothing on crates.io wraps PDF/A-3 attachment
//! extraction any higher than that, so this module is the walk from
//! `Root` down to the embedded file stream: `Root/Names/EmbeddedFiles` is
//! a name tree (`[name, filespec_ref, name, filespec_ref, ...]`); each
//! filespec's `/EF` dictionary points at the actual stream, under `/F`
//! (or `/UF` for the Unicode variant some producers use instead).

use lopdf::{Dictionary, Document, Object};

use crate::domain::supplier_invoice::ports::SupplierInvoiceParseError;

/// Attachment names real Factur-X/ZUGFeRD producers use for the CII
/// document, checked case-insensitively. Preferred over "whichever
/// attachment comes first" because a PDF/A-3 invoice can legally carry
/// other attachments alongside it (a logo, a delivery note); falling back
/// to the sole attachment (or the first of several) only when none of
/// these names match keeps single-attachment producers working too.
const KNOWN_CII_ATTACHMENT_NAMES: [&str; 4] = [
    "factur-x.xml",
    "facturx.xml",
    "zugferd-invoice.xml",
    "xrechnung.xml",
];

pub(super) fn extract_embedded_xml(bytes: &[u8]) -> Result<Vec<u8>, SupplierInvoiceParseError> {
    let doc = Document::load_mem(bytes)
        .map_err(|e| attachment_error(format!("not a readable PDF: {e}")))?;

    let embedded_files = embedded_files_dict(&doc)?;
    let names = embedded_files
        .get_deref(b"Names", &doc)
        .and_then(Object::as_array)
        .map_err(|e| attachment_error(format!("no attachment name tree: {e}")))?;

    let candidates = collect_filespecs(&doc, names);
    let (_, filespec) = candidates
        .iter()
        .find(|(name, _)| is_known_cii_name(name))
        .or_else(|| candidates.first())
        .ok_or_else(|| attachment_error("PDF has no embedded files".to_owned()))?;

    read_filespec_content(&doc, filespec)
}

fn embedded_files_dict(doc: &Document) -> Result<&Dictionary, SupplierInvoiceParseError> {
    let catalog = doc
        .catalog()
        .map_err(|e| attachment_error(format!("no document catalog: {e}")))?;

    let names = catalog
        .get_deref(b"Names", doc)
        .and_then(Object::as_dict)
        .map_err(|e| attachment_error(format!("no /Names dictionary: {e}")))?;

    names
        .get_deref(b"EmbeddedFiles", doc)
        .and_then(Object::as_dict)
        .map_err(|e| attachment_error(format!("no /EmbeddedFiles entry: {e}")))
}

/// The name tree alternates a PDF string (the attachment's file name) and
/// an indirect reference to its file specification dictionary. Any pair
/// this document's own irregularities keep from resolving is skipped, not
/// fatal — one broken entry should not hide every other attachment.
fn collect_filespecs<'a>(doc: &'a Document, names: &'a [Object]) -> Vec<(String, &'a Dictionary)> {
    names
        .chunks(2)
        .filter_map(|pair| {
            let [name_obj, filespec_obj] = pair else {
                return None;
            };
            let name = name_obj
                .as_str()
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                .ok()?;
            let (_, resolved) = doc.dereference(filespec_obj).ok()?;
            let filespec = resolved.as_dict().ok()?;

            Some((name, filespec))
        })
        .collect()
}

fn is_known_cii_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    KNOWN_CII_ATTACHMENT_NAMES
        .iter()
        .any(|known| lower == *known)
}

fn read_filespec_content(
    doc: &Document,
    filespec: &Dictionary,
) -> Result<Vec<u8>, SupplierInvoiceParseError> {
    let ef = filespec
        .get_deref(b"EF", doc)
        .and_then(Object::as_dict)
        .map_err(|e| attachment_error(format!("filespec has no /EF dictionary: {e}")))?;

    let file_ref = ef
        .get(b"F")
        .or_else(|_| ef.get(b"UF"))
        .map_err(|e| attachment_error(format!("/EF has neither /F nor /UF: {e}")))?;

    let (_, stream_obj) = doc
        .dereference(file_ref)
        .map_err(|e| attachment_error(format!("embedded file stream missing: {e}")))?;
    let stream = stream_obj
        .as_stream()
        .map_err(|e| attachment_error(format!("embedded file is not a stream: {e}")))?;

    stream
        .decompressed_content()
        .or_else(|_| Ok(stream.content.clone()))
}

fn attachment_error(reason: String) -> SupplierInvoiceParseError {
    SupplierInvoiceParseError::AttachmentExtraction(reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_PDF: &[u8] = include_bytes!("fixtures/valid.pdf");
    const MALFORMED_PDF: &[u8] = include_bytes!("fixtures/malformed.pdf");

    #[test]
    fn extracts_the_embedded_cii_xml_from_a_real_factur_x_pdf() {
        let xml = extract_embedded_xml(VALID_PDF).expect("a valid Factur-X PDF");

        let xml = String::from_utf8(xml).expect("CII XML is UTF-8");
        assert!(xml.contains("CrossIndustryInvoice"));
        assert!(xml.contains("F20260023"));
    }

    #[test]
    fn refuses_a_truncated_pdf_as_an_attachment_extraction_error() {
        let error = extract_embedded_xml(MALFORMED_PDF).unwrap_err();

        assert!(matches!(
            error,
            SupplierInvoiceParseError::AttachmentExtraction(_)
        ));
    }

    #[test]
    fn refuses_a_pdf_with_no_names_dictionary_at_all() {
        let plain_pdf = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<< /Type /Catalog >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF";

        let error = extract_embedded_xml(plain_pdf).unwrap_err();

        assert!(matches!(
            error,
            SupplierInvoiceParseError::AttachmentExtraction(_)
        ));
    }
}
