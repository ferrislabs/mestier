use auth::Identity;
use axum::{
    Extension,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use handlers::{ApiError, AppState};
use http::{
    StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use mestier_core::{Quote, QuoteId};

use crate::{paths::QuotePdfPath, require_quote_membership};

#[utoipa::path(
    get,
    path = "/api/v1/quotes/{quote_id}/pdf",
    operation_id = "exportQuotePdf",
    tag = super::super::TAG,
    params(
        ("quote_id" = QuoteId, Path, description = "Quote identifier"),
    ),
    responses(
        (status = 200, description = "Quote PDF export", content_type = "application/pdf"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuotePdfPath { quote_id }: QuotePdfPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response, ApiError> {
    let quote = require_quote_membership(&state, &identity, quote_id).await?;
    let pdf = render_quote_pdf(&quote);
    let filename = format!("{}.pdf", quote.reference);

    Ok((
        StatusCode::OK,
        [
            (CONTENT_TYPE, "application/pdf".to_owned()),
            (
                CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        Body::from(pdf),
    )
        .into_response())
}

fn render_quote_pdf(quote: &Quote) -> Vec<u8> {
    let mut text_lines = vec![
        "Mestier - Devis".to_owned(),
        format!("Reference: {}", quote.reference),
        format!("Objet: {}", quote.title),
        format!("Identifiant technique: {}", quote.id.0),
        format!("Client: {}", quote.customer_id.0),
        format!("Contexte: {}", quote.customer_context_id.0),
        format!("Statut: {}", quote.status.as_str()),
        format!("Total: {}", format_cents(quote.gross_cents)),
        String::new(),
        "Lignes".to_owned(),
    ];

    for (index, line) in quote.lines.iter().enumerate() {
        text_lines.push(format!(
            "{}. {} - {} x {} = {}",
            index + 1,
            line.label,
            line.quantity.normalize(),
            format_cents(line.unit_price_cents),
            format_cents(line_total_cents(line.quantity, line.unit_price_cents)),
        ));

        if let Some(notes) = &line.notes {
            text_lines.push(format!("   Notes: {notes}"));
        }

        if !line.photo_keys.is_empty() {
            text_lines.push(format!("   Photos: {}", line.photo_keys.join(", ")));
        }
    }

    let content = render_pdf_text_stream(&text_lines);
    build_pdf(content.as_bytes())
}

fn line_total_cents(quantity: rust_decimal::Decimal, unit_price_cents: i32) -> i32 {
    use rust_decimal::prelude::ToPrimitive;

    (quantity * rust_decimal::Decimal::from(unit_price_cents))
        .round_dp(0)
        .to_i32()
        .unwrap_or(0)
}

fn format_cents(cents: i32) -> String {
    let euros = cents / 100;
    let remainder = cents.abs() % 100;
    format!("{euros}.{remainder:02} EUR")
}

fn render_pdf_text_stream(lines: &[String]) -> String {
    let mut stream = String::from("BT\n/F1 12 Tf\n50 790 Td\n14 TL\n");

    for line in lines {
        stream.push('(');
        stream.push_str(&escape_pdf_text(line));
        stream.push_str(") Tj\nT*\n");
    }

    stream.push_str("ET\n");
    stream
}

fn escape_pdf_text(input: &str) -> String {
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

fn build_pdf(content: &[u8]) -> Vec<u8> {
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
