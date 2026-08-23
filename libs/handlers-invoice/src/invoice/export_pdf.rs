use auth::Identity;
use axum::{
    Extension,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use handlers::{ApiError, AppState};
use handlers_quote::pdf::{
    build_pdf, format_cents, format_cents_i64, format_rate_bp, render_pdf_text_stream,
    vat_status_mention,
};
use http::{
    StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use mestier_core::{Customer, Invoice, InvoiceId, LegalIdentity, Organization, VatStatus};

use crate::{paths::InvoicePdfPath, require_invoice_membership};

#[utoipa::path(
    get,
    path = "/api/v1/invoices/{invoice_id}/pdf",
    operation_id = "exportInvoicePdf",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    responses(
        (status = 200, description = "Invoice PDF export", content_type = "application/pdf"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
        (status = 409, description = "The organization's legal identity is incomplete; the response names every missing field"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoicePdfPath { invoice_id }: InvoicePdfPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response, ApiError> {
    let invoice = require_invoice_membership(&state, &identity, invoice_id).await?;
    let organization = state
        .usecase
        .get_organization(invoice.organization_id)
        .await?;
    let customer = state.usecase.get_customer(invoice.customer_id).await?;

    // Refuse rather than render a blank: a PDF with an empty SIRET line
    // looks fine and is not. Same gate, same message shape, as
    // `handlers_quote::quote::export_pdf` — the settings section (#311) is
    // where the artisan goes to fix whatever is named here.
    let legal_identity =
        LegalIdentity::try_from_organization(&organization).map_err(|missing| {
            ApiError::Conflict(format!(
                "cannot issue a document: the organization's legal identity is missing {}",
                missing.join(", ")
            ))
        })?;

    let pdf = render_invoice_pdf(&invoice, &legal_identity, &organization, &customer);
    let filename_stem = invoice
        .number
        .clone()
        .unwrap_or_else(|| invoice.id.0.to_string());
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

/// Renders the figures the domain already computed; it must not compute
/// any of its own, per `CLAUDE.md` — a total the PDF derives independently
/// is a total that can disagree with the one the screen shows. The
/// low-level machinery (`render_pdf_text_stream`, `build_pdf`,
/// `format_cents`, `format_rate_bp`, `vat_status_mention`) is
/// `handlers_quote::pdf`, shared verbatim with the quote export — only
/// which lines of text get written differs, because an invoice carries
/// fields a quote does not (`kind`, `number`, `issued_at`, the e-invoicing
/// mentions #341 added).
fn render_invoice_pdf(
    invoice: &Invoice,
    identity: &LegalIdentity,
    organization: &Organization,
    customer: &Customer,
) -> Vec<u8> {
    let title = match invoice.kind {
        mestier_core::InvoiceKind::CreditNote => "Mestier - Avoir",
        _ => "Mestier - Facture",
    };

    let mut text_lines = vec![
        title.to_owned(),
        format!("N. {}", invoice.number.as_deref().unwrap_or("(brouillon)")),
        format!(
            "Date d'emission: {}",
            invoice
                .issued_at
                .unwrap_or(invoice.created_at)
                .format("%d/%m/%Y")
        ),
    ];

    if let Some(due_at) = invoice.due_at {
        text_lines.push(format!("Date d'echeance: {}", due_at.format("%d/%m/%Y")));
    }
    if let Some(source_invoice_id) = invoice.source_invoice_id {
        text_lines.push(format!("Avoir sur facture: {source_invoice_id}"));
    }

    text_lines.push(String::new());
    text_lines.push("Emetteur".to_owned());
    text_lines.push(organization.name.clone());
    text_lines.push(identity.legal_name.clone());
    text_lines.push(format!(
        "{} - SIRET {}",
        identity.legal_form, identity.registration_number
    ));
    text_lines.push(identity.address.line1.clone());
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
    // The e-invoicing reform's own customer-side mention (#341): the
    // customer's SIREN, printed when known.
    if let Some(registration_number) = &customer.registration_number {
        text_lines.push(format!("SIREN client: {registration_number}"));
    }

    if let Some(operation_nature) = invoice.operation_nature {
        text_lines.push(format!(
            "Nature de l'operation: {}",
            operation_nature_label(operation_nature)
        ));
    }

    if let Some(delivery_address) = &invoice.delivery_address {
        text_lines.push(String::new());
        text_lines.push("Lieu de livraison".to_owned());
        text_lines.push(delivery_address.line1.clone());
        if let Some(line2) = &delivery_address.line2 {
            text_lines.push(line2.clone());
        }
        text_lines.push(format!(
            "{} {}, {}",
            delivery_address.postal_code, delivery_address.city, delivery_address.country
        ));
    }

    text_lines.push(String::new());
    text_lines.push("Lignes".to_owned());

    let subject_to_vat = matches!(identity.vat_status, VatStatus::Subject { .. });

    for (index, line) in invoice.lines.iter().enumerate() {
        let rate = if subject_to_vat {
            format!(
                ", TVA {}",
                format_rate_bp(line.vat_rate_basis_points.unwrap_or(0))
            )
        } else {
            String::new()
        };

        text_lines.push(format!(
            "{}. {} - {} x {}{} = {}",
            index + 1,
            line.label,
            line.quantity.normalize(),
            format_cents(line.unit_price_cents),
            rate,
            format_cents(line_net_cents(line.quantity, line.unit_price_cents)),
        ));
    }

    text_lines.push(String::new());
    text_lines.push("Totaux".to_owned());
    text_lines.push(format!("Total HT: {}", format_cents(invoice.net_cents)));

    if invoice.vat_breakdown.is_empty() {
        // Never a breakdown of zeros: this line only appears for an
        // organization that stated it charges no VAT, and says why.
        if let VatStatus::NotSubject { basis } = &identity.vat_status {
            text_lines.push(format!("TVA non applicable, {basis}"));
        }
    } else {
        for breakdown in &invoice.vat_breakdown {
            text_lines.push(format!(
                "TVA {}: {}",
                format_rate_bp(breakdown.rate_bp),
                format_cents(breakdown.vat_cents)
            ));
        }
    }

    text_lines.push(format!("Total TTC: {}", format_cents(invoice.gross_cents)));

    let content = render_pdf_text_stream(&text_lines);
    build_pdf(content.as_bytes())
}

fn operation_nature_label(operation_nature: mestier_core::OperationNature) -> &'static str {
    match operation_nature {
        mestier_core::OperationNature::Goods => "Livraison de biens",
        mestier_core::OperationNature::Services => "Prestation de services",
        mestier_core::OperationNature::Both => "Livraison de biens et prestation de services",
    }
}

fn line_net_cents(quantity: rust_decimal::Decimal, unit_price_cents: i32) -> i32 {
    use rust_decimal::prelude::ToPrimitive;

    (quantity * rust_decimal::Decimal::from(unit_price_cents))
        .round_dp(0)
        .to_i32()
        .unwrap_or(0)
}
