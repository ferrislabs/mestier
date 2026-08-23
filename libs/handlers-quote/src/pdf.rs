//! The low-level PDF/text machinery shared by every document export in the
//! commercial area: issuer block, customer block, VAT breakdown and legal
//! mentions are laid out the same way on a quote (this crate) and an
//! invoice (`handlers-invoice`, #319) — two renderers would mean those
//! mentions get fixed on only one of them. This module is the one
//! implementation; what text ends up on the page is still each document's
//! own business (`quote::export_pdf::render_quote_pdf`,
//! `handlers_invoice::invoice::export_pdf::render_invoice_pdf`).
//!
//! No external PDF crate: a handful of fixed objects and a content stream
//! of `Tj`/`T*` operators is enough for a single-page text document, and it
//! keeps the dependency graph free of a rendering engine for something this
//! small.

use mestier_core::VatStatus;

/// The mention a document must carry either way: the VAT number when the
/// organization charges VAT, the legal basis for the exemption when it does
/// not. Never a blank — see #310.
pub fn vat_status_mention(status: &VatStatus) -> String {
    match status {
        VatStatus::Subject { vat_number } => format!("N. TVA intracommunautaire: {vat_number}"),
        VatStatus::NotSubject { basis } => format!("TVA non applicable, {basis}"),
    }
}

/// Basis points to a percentage string: 2000 -> "20.00%", 550 -> "5.50%".
pub fn format_rate_bp(rate_bp: i32) -> String {
    let whole = rate_bp / 100;
    let hundredths = rate_bp % 100;
    format!("{whole}.{hundredths:02}%")
}

pub fn format_cents(cents: i32) -> String {
    format_cents_i64(i64::from(cents))
}

pub fn format_cents_i64(cents: i64) -> String {
    let euros = cents / 100;
    let remainder = cents.abs() % 100;
    format!("{euros}.{remainder:02} EUR")
}

pub fn render_pdf_text_stream(lines: &[String]) -> String {
    let mut stream = String::from("BT\n/F1 10 Tf\n40 800 Td\n12 TL\n");

    for line in lines {
        stream.push('(');
        stream.push_str(&escape_pdf_text(line));
        stream.push_str(") Tj\nT*\n");
    }

    stream.push_str("ET\n");
    stream
}

pub fn escape_pdf_text(input: &str) -> String {
    input
        .chars()
        .map(|ch| match ch {
            '(' => "\\(".to_owned(),
            ')' => "\\)".to_owned(),
            '\\' => "\\\\".to_owned(),
            ch if ch.is_ascii() && !ch.is_control() => ch.to_string(),
            _ => "?".to_owned(),
        })
        .collect()
}

pub fn build_pdf(content: &[u8]) -> Vec<u8> {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_owned(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        format!(
            "<< /Length {} >>\nstream\n{}endstream",
            content.len(),
            String::from_utf8_lossy(content)
        ),
    ];

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");

    let mut offsets = Vec::with_capacity(objects.len());
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
    }

    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }

    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );

    pdf
}
