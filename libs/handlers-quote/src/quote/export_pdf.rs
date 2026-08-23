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
use mestier_core::{Customer, LegalIdentity, Organization, Quote, QuoteId, VatStatus};

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
        (status = 409, description = "The organization's legal identity is incomplete; the response names every missing field"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuotePdfPath { quote_id }: QuotePdfPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response, ApiError> {
    let quote = require_quote_membership(&state, &identity, quote_id).await?;
    let organization = state
        .usecase
        .get_organization(quote.organization_id)
        .await?;
    let customer = state.usecase.get_customer(quote.customer_id).await?;

    // Refuse rather than render a blank: a PDF with an empty SIRET line
    // looks fine and is not. The settings section (#311) is where the
    // artisan goes to fix whatever is named here.
    let identity = LegalIdentity::try_from_organization(&organization).map_err(|missing| {
        ApiError::Conflict(format!(
            "cannot issue a document: the organization's legal identity is missing {}",
            missing.join(", ")
        ))
    })?;

    let pdf = render_quote_pdf(&quote, &identity, &organization, &customer);
    let filename_stem = quote
        .reference
        .clone()
        .unwrap_or_else(|| quote.id.0.to_string());
    let filename = format!("{filename_stem}.pdf");

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

/// Renders the figures the domain already computed; it must not compute any
/// of its own, per `CLAUDE.md` — a total the PDF derives independently is a
/// total that can disagree with the one the screen shows.
fn render_quote_pdf(
    quote: &Quote,
    identity: &LegalIdentity,
    organization: &Organization,
    customer: &Customer,
) -> Vec<u8> {
    let mut text_lines = vec![
        "Mestier - Devis".to_owned(),
        format!("N. {}", quote.reference.as_deref().unwrap_or("(brouillon)")),
        format!("Date: {}", quote.created_at.format("%d/%m/%Y")),
        String::new(),
        "Emetteur".to_owned(),
        organization.name.clone(),
        identity.legal_name.clone(),
        format!(
            "{} - SIRET {}",
            identity.legal_form, identity.registration_number
        ),
        identity.address.line1.clone(),
    ];

    if let Some(line2) = &identity.address.line2 {
        text_lines.push(line2.clone());
    }
    text_lines.push(format!(
        "{} {}, {}",
        identity.address.postal_code, identity.address.city, identity.address.country
    ));
    text_lines.push(vat_status_mention(&identity.vat_status));
    if let Some(capital) = identity.share_capital_cents {
        text_lines.push(format!("Capital social: {}", format_cents_i64(capital)));
    }
    text_lines.push(format!(
        "Assurance professionnelle: {}",
        identity.insurance_mention
    ));
    if let Some(email) = &identity.contact_email {
        text_lines.push(format!("Email: {email}"));
    }
    if let Some(phone) = &identity.contact_phone {
        text_lines.push(format!("Tel: {phone}"));
    }

    text_lines.push(String::new());
    text_lines.push("Client".to_owned());
    text_lines.push(customer.name.clone());

    text_lines.push(String::new());
    text_lines.push(format!("Objet: {}", quote.title));
    text_lines.push(String::new());
    text_lines.push("Lignes".to_owned());

    let subject_to_vat = matches!(identity.vat_status, VatStatus::Subject { .. });

    for (index, line) in quote.lines.iter().enumerate() {
        let rate = if subject_to_vat {
            format!(", TVA {}", format_rate_bp(line.vat_rate_bp.unwrap_or(0)))
        } else {
            String::new()
        };

        text_lines.push(format!(
            "{}. {} - {} {} x {}{} = {}",
            index + 1,
            line.label,
            line.quantity.normalize(),
            line.unit.as_str(),
            format_cents(line.unit_price_cents),
            rate,
            format_cents(line_net_cents(line.quantity, line.unit_price_cents)),
        ));

        if let Some(notes) = &line.notes {
            text_lines.push(format!("   Notes: {notes}"));
        }

        if !line.photo_keys.is_empty() {
            text_lines.push(format!("   Photos: {}", line.photo_keys.join(", ")));
        }
    }

    text_lines.push(String::new());
    text_lines.push("Totaux".to_owned());
    text_lines.push(format!("Total HT: {}", format_cents(quote.net_cents)));

    if quote.vat_breakdown.is_empty() {
        // Never a breakdown of zeros: this line only appears for an
        // organization that stated it charges no VAT, and says why.
        if let VatStatus::NotSubject { basis } = &identity.vat_status {
            text_lines.push(format!("TVA non applicable, {basis}"));
        }
    } else {
        for breakdown in &quote.vat_breakdown {
            text_lines.push(format!(
                "TVA {}: {}",
                format_rate_bp(breakdown.rate_bp),
                format_cents(breakdown.vat_cents)
            ));
        }
    }

    text_lines.push(format!("Total TTC: {}", format_cents(quote.gross_cents)));

    let content = render_pdf_text_stream(&text_lines);
    build_pdf(content.as_bytes())
}

/// The mention a document must carry either way: the VAT number when the
/// organization charges VAT, the legal basis for the exemption when it does
/// not. Never a blank — see #310.
fn vat_status_mention(status: &VatStatus) -> String {
    match status {
        VatStatus::Subject { vat_number } => format!("N. TVA intracommunautaire: {vat_number}"),
        VatStatus::NotSubject { basis } => format!("TVA non applicable, {basis}"),
    }
}

/// Basis points to a percentage string: 2000 -> "20.00%", 550 -> "5.50%".
fn format_rate_bp(rate_bp: i32) -> String {
    let whole = rate_bp / 100;
    let hundredths = rate_bp % 100;
    format!("{whole}.{hundredths:02}%")
}

fn line_net_cents(quantity: rust_decimal::Decimal, unit_price_cents: i32) -> i32 {
    use rust_decimal::prelude::ToPrimitive;

    (quantity * rust_decimal::Decimal::from(unit_price_cents))
        .round_dp(0)
        .to_i32()
        .unwrap_or(0)
}

fn format_cents(cents: i32) -> String {
    format_cents_i64(i64::from(cents))
}

fn format_cents_i64(cents: i64) -> String {
    let euros = cents / 100;
    let remainder = cents.abs() % 100;
    format!("{euros}.{remainder:02} EUR")
}

fn render_pdf_text_stream(lines: &[String]) -> String {
    let mut stream = String::from("BT\n/F1 10 Tf\n40 800 Td\n12 TL\n");

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
