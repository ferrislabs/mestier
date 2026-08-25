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

use crate::{
    paths::QuotePdfPath,
    pdf::{
        build_pdf, format_cents, format_cents_i64, format_rate_bp, render_pdf_text_stream,
        vat_status_mention,
    },
    require_quote_membership,
};

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

fn line_net_cents(quantity: rust_decimal::Decimal, unit_price_cents: i32) -> i32 {
    use rust_decimal::prelude::ToPrimitive;

    (quantity * rust_decimal::Decimal::from(unit_price_cents))
        .round_dp(0)
        .to_i32()
        .unwrap_or(0)
}
