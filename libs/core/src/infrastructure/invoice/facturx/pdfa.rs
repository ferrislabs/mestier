//! Embeds a CII XML payload into a rendered PDF as a Factur-X-shaped
//! PDF/A-3 container: the half `en16931-formats` deliberately does not
//! provide (see `infrastructure::invoice::facturx`'s own module doc
//! comment, and `en16931_formats::zugferd`'s "Writing: out of scope, and
//! not for want of effort").
//!
//! This walks the same `lopdf` object model
//! `infrastructure::supplier_invoice::facturx::attachment` already walks to
//! *read* an embedded file, in reverse, to *write* one:
//! `Root/Names/EmbeddedFiles` (so a reader without PDF/A-3 support still
//! finds the attachment by name), `Root/AF` (so a PDF/A-3-aware reader
//! finds it as an **associated** file, not merely an attached one — "the
//! commonest defect: every PDF library can attach a file, fewer can
//! associate one", per `en16931-formats`'s own module documentation), and a
//! `/Metadata` XMP packet declaring the PDF/A and Factur-X profile.
//!
//! # What this cannot verify from inside this sandbox, and why the XMP
//! claim below is honest about that rather than silent
//!
//! ISO 19005-3 (PDF/A-3) requires, among many other things, **every font
//! embedded** — including the 14 "standard" fonts a plain PDF may reference
//! by name alone. `handlers_quote::pdf::build_pdf` (the renderer whose
//! output this module receives as `visual_document`) draws its text with
//! `/BaseFont /Helvetica` and embeds nothing: correct and sufficient for a
//! plain PDF, and **not** PDF/A conformant on its own, in a way this module
//! cannot fix — fixing it means the renderer embeds a font program, which
//! is `handlers-quote`'s file, not this adapter's, and out of #342's own
//! scope.
//!
//! This module still writes `pdfaid:part = 3` / `pdfaid:conformance = B`
//! into the XMP, because a hybrid PDF with no such claim is not
//! recognisable as Factur-X *at all* — "the XMP is not decoration. It is
//! how a receiver discovers an invoice is there", per the same crate quoted
//! above. **That claim is not verified.** Nothing available here can run
//! the ISO 19005-3 conformance check a receiving platform will actually
//! apply (veraPDF is the tool; it is not installed, and there is no network
//! path to one that is). Treat every file this module produces as
//! Factur-X-*shaped* — correct embedding, correct association, a genuinely
//! validated CII payload — until a human runs it through veraPDF (or
//! Chorus Pro's own test submission) and either confirms the PDF/A-3 claim
//! or reports which check failed, at which point the fix most likely
//! belongs in the renderer, not here.

use lopdf::{Dictionary, Document, Object, ObjectId, Stream, StringFormat};

use crate::domain::invoice::ports::DocumentFormatError;

/// Attachment name a Factur-X writer picks — see
/// `infrastructure::supplier_invoice::facturx::attachment`'s own
/// `KNOWN_CII_ATTACHMENT_NAMES` for why a *reader* accepts several and a
/// *writer* picks one.
pub(super) const CII_ATTACHMENT_NAME: &str = "factur-x.xml";

/// `/AFRelationship` for a profile that carries lines (ours always does —
/// #342 never generates MINIMUM or BASIC WL). Published guidance
/// disagrees on the replacement for `Data` (wrong for a lines-carrying
/// profile per every source `en16931-formats` cites): German sources say
/// `Alternative`, PDFlib documents `Source` for Factur-X shipped to
/// non-German recipients. `Alternative` is what the Factur-X 1.0 reference
/// info package's own sample PDFs use, which is the closest thing to an
/// authority available here; a human confirming this against Chorus Pro's
/// own accepted convention is named explicitly in this issue's own report.
const AF_RELATIONSHIP: &str = "Alternative";

/// Embeds `cii_xml` into `visual_pdf`, producing the file this invoice's
/// `document_file_key` will point at.
///
/// Deterministic for the same `(visual_pdf, cii_xml, invoice_id,
/// issued_at)`: no wall-clock read anywhere in this function, no random
/// `/ID` — see [`deterministic_file_id`].
pub(super) fn embed_cii_xml(
    visual_pdf: &[u8],
    cii_xml: &str,
    invoice_id: uuid::Uuid,
    issued_at: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<u8>, DocumentFormatError> {
    let mut doc = Document::load_mem(visual_pdf)
        .map_err(|e| DocumentFormatError::Embedding(format!("not a readable PDF: {e}")))?;

    let xml_bytes = cii_xml.as_bytes().to_vec();
    let ef_stream_id = doc.add_object(Object::Stream(embedded_file_stream(xml_bytes)));

    let filespec_dict = filespec(ef_stream_id);
    let filespec_id = doc.add_object(Object::Dictionary(filespec_dict));

    let metadata_id = doc.add_object(Object::Stream(metadata_stream(issued_at)));

    {
        let catalog = doc
            .catalog_mut()
            .map_err(|e| DocumentFormatError::Embedding(format!("no document catalog: {e}")))?;

        let mut embedded_files = Dictionary::new();
        embedded_files.set(
            "Names",
            Object::Array(vec![
                Object::String(
                    CII_ATTACHMENT_NAME.as_bytes().to_vec(),
                    StringFormat::Literal,
                ),
                Object::Reference(filespec_id),
            ]),
        );
        let mut names = Dictionary::new();
        names.set("EmbeddedFiles", Object::Dictionary(embedded_files));
        catalog.set("Names", Object::Dictionary(names));

        // The associated-files array: what makes this an *associated* file
        // under PDF/A-3, not merely an attached one. See this module's own
        // doc comment.
        catalog.set("AF", Object::Array(vec![Object::Reference(filespec_id)]));
        catalog.set("Metadata", Object::Reference(metadata_id));
    }

    // A deterministic file `/ID`, derived from the invoice rather than
    // wall-clock time or randomness — `lopdf`'s writer does not generate
    // one itself (unlike `Document::with_version`'s sibling helpers in
    // `creator.rs`, which this path never calls), so an omitted `/ID` would
    // simply be absent, not random; it is set explicitly both because a
    // PDF/A file is expected to carry one and to keep that expectation from
    // ever becoming a footgun if a future refactor reaches for a
    // `creator.rs` helper that does generate one randomly.
    let id_bytes = invoice_id.as_bytes().to_vec();
    doc.trailer.set(
        "ID",
        Object::Array(vec![
            Object::String(id_bytes.clone(), StringFormat::Hexadecimal),
            Object::String(id_bytes, StringFormat::Hexadecimal),
        ]),
    );

    let mut out = Vec::new();
    doc.save_to(&mut out)
        .map_err(|e| DocumentFormatError::Embedding(format!("could not write the PDF: {e}")))?;
    Ok(out)
}

fn embedded_file_stream(xml_bytes: Vec<u8>) -> Stream {
    let mut params = Dictionary::new();
    params.set("Size", Object::Integer(xml_bytes.len() as i64));

    let mut dict = Dictionary::new();
    dict.set("Type", Object::Name(b"EmbeddedFile".to_vec()));
    dict.set("Subtype", Object::Name(b"text/xml".to_vec()));
    dict.set("Params", Object::Dictionary(params));

    Stream::new(dict, xml_bytes)
}

fn filespec(ef_stream_id: ObjectId) -> Dictionary {
    let mut ef = Dictionary::new();
    ef.set("F", Object::Reference(ef_stream_id));
    ef.set("UF", Object::Reference(ef_stream_id));

    let mut dict = Dictionary::new();
    dict.set("Type", Object::Name(b"Filespec".to_vec()));
    dict.set(
        "F",
        Object::String(
            CII_ATTACHMENT_NAME.as_bytes().to_vec(),
            StringFormat::Literal,
        ),
    );
    dict.set(
        "UF",
        Object::String(
            CII_ATTACHMENT_NAME.as_bytes().to_vec(),
            StringFormat::Literal,
        ),
    );
    dict.set(
        "Desc",
        Object::String(b"Factur-X CII invoice data".to_vec(), StringFormat::Literal),
    );
    dict.set("EF", Object::Dictionary(ef));
    dict.set(
        "AFRelationship",
        Object::Name(AF_RELATIONSHIP.as_bytes().to_vec()),
    );
    dict
}

/// The XMP packet — see this module's own doc comment on what the PDF/A-3
/// claim inside it does and does not mean.
///
/// The namespace URI's mixed case and trailing `#` are both load-bearing —
/// `en16931_formats::zugferd`'s own module doc comment flags this as one of
/// "two things a writer gets wrong first" — and there is exactly one
/// `rdf:Description` per schema below, so the "two schemas collide in one
/// `pdfaExtension:schemas` bag" trap that same doc comment describes does
/// not apply here: this packet is written from scratch, not merged into an
/// existing one.
fn metadata_stream(issued_at: chrono::DateTime<chrono::Utc>) -> Stream {
    use chrono::Datelike;
    // The invoice's own issue date, not `Utc::now()`: a wall-clock read
    // here would make two generations of the same invoice byte-different,
    // which is exactly the property #342's determinism test exists to
    // catch.
    let date = format!(
        "{:04}-{:02}-{:02}",
        issued_at.year(),
        issued_at.month(),
        issued_at.day()
    );

    let xml = format!(
        r#"<?xpacket begin="﻿" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about=""
        xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/">
      <pdfaid:part>3</pdfaid:part>
      <pdfaid:conformance>B</pdfaid:conformance>
    </rdf:Description>
    <rdf:Description rdf:about=""
        xmlns:fx="urn:factur-x:pdfa:CrossIndustryDocument:invoice:1p0#">
      <fx:DocumentType>INVOICE</fx:DocumentType>
      <fx:DocumentFileName>{attachment_name}</fx:DocumentFileName>
      <fx:Version>1.0</fx:Version>
      <fx:ConformanceLevel>EN 16931</fx:ConformanceLevel>
    </rdf:Description>
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:format>application/pdf</dc:format>
      <dc:date>{date}</dc:date>
    </rdf:Description>
    <rdf:Description rdf:about=""
        xmlns:pdf="http://ns.adobe.com/pdf/1.3/">
      <pdf:Producer>Mestier</pdf:Producer>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
        attachment_name = CII_ATTACHMENT_NAME,
    );

    let mut dict = Dictionary::new();
    dict.set("Type", Object::Name(b"Metadata".to_vec()));
    dict.set("Subtype", Object::Name(b"XML".to_vec()));
    Stream::new(dict, xml.into_bytes())
}
