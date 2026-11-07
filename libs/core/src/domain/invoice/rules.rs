use std::collections::BTreeMap;

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use common::generate_uuid_v7;
use rust_decimal::{Decimal, RoundingStrategy, prelude::ToPrimitive};

use super::{
    Invoice, InvoiceDisplayState, InvoiceError, InvoiceField, InvoiceId, InvoiceKind, InvoiceLine,
    InvoiceLineId, InvoiceOutcome, InvoiceSettlement, InvoiceStatus, InvoiceVatBreakdownLine,
    commands::{CreditLineRequest, CreditRequest},
};
use crate::{
    CustomerKind, DepositPercent, DiscountPercent, OrganizationId, Quote, QuoteStatus,
    domain::{organization::billing_settings::BillingSettings, quote::pricing::vat_cents},
};

fn out_of_bounds() -> InvoiceError {
    InvoiceError::invalid(InvoiceField::Lines, "the total is outside supported bounds")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoicePricing {
    pub net_cents: i32,
    pub vat_breakdown: Vec<InvoiceVatBreakdownLine>,
    pub gross_cents: i32,
}

pub fn line_net_cents(line: &InvoiceLine) -> InvoiceOutcome<i64> {
    let remaining = Decimal::ONE_HUNDRED - line.discount_percent.get();
    let net = line
        .quantity
        .checked_mul(Decimal::from(line.unit_price_cents))
        .and_then(|amount| amount.checked_mul(remaining))
        .map(|amount| amount / Decimal::ONE_HUNDRED)
        .ok_or_else(out_of_bounds)?;

    net.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .ok_or_else(out_of_bounds)
}

pub fn net_by_rate(lines: &[InvoiceLine]) -> InvoiceOutcome<BTreeMap<i32, i64>> {
    let mut bases: BTreeMap<i32, i64> = BTreeMap::new();

    for line in lines {
        let base = bases
            .entry(line.vat_rate_basis_points.unwrap_or(0))
            .or_insert(0);
        *base = base
            .checked_add(line_net_cents(line)?)
            .ok_or_else(out_of_bounds)?;
    }

    Ok(bases)
}

fn to_i32(value: i64) -> InvoiceOutcome<i32> {
    i32::try_from(value).map_err(|_| out_of_bounds())
}

pub fn totals_of(lines: &[InvoiceLine], subject_to_vat: bool) -> InvoiceOutcome<InvoicePricing> {
    let bases = net_by_rate(lines)?;
    let net: i64 = bases.values().sum();

    let vat_by_rate = if subject_to_vat {
        bases
            .into_iter()
            .map(|(rate_bp, base)| {
                vat_cents(base, rate_bp)
                    .map(|vat| (rate_bp, vat))
                    .map_err(|_| out_of_bounds())
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    let vat_total: i64 = vat_by_rate.iter().map(|(_, vat)| vat).sum();

    Ok(InvoicePricing {
        net_cents: to_i32(net)?,
        vat_breakdown: vat_by_rate
            .into_iter()
            .map(|(rate_bp, vat)| {
                to_i32(vat).map(|vat_cents| InvoiceVatBreakdownLine { rate_bp, vat_cents })
            })
            .collect::<Result<Vec<_>, _>>()?,
        gross_cents: to_i32(net.checked_add(vat_total).ok_or_else(out_of_bounds)?)?,
    })
}

pub fn settlement_of(
    status: InvoiceStatus,
    kind: InvoiceKind,
    gross_cents: i32,
    credited_cents: i64,
    paid_cents: i64,
) -> InvoiceSettlement {
    let balance_cents = match (status, kind) {
        (InvoiceStatus::Draft, _) | (_, InvoiceKind::CreditNote) => 0,
        (InvoiceStatus::Cancelled, _) => -paid_cents,
        _ => i64::from(gross_cents) - credited_cents - paid_cents,
    };

    InvoiceSettlement {
        paid_cents,
        credited_cents,
        balance_cents,
    }
}

pub fn display_state(
    status: InvoiceStatus,
    kind: InvoiceKind,
    gross_cents: i32,
    due_on: Option<NaiveDate>,
    settlement: InvoiceSettlement,
    today: NaiveDate,
) -> InvoiceDisplayState {
    if status == InvoiceStatus::Draft {
        return InvoiceDisplayState::Draft;
    }
    if kind == InvoiceKind::CreditNote {
        return InvoiceDisplayState::Paid;
    }
    if settlement.balance_cents < 0 {
        return InvoiceDisplayState::RefundDue;
    }
    if status == InvoiceStatus::Cancelled
        || (gross_cents > 0 && settlement.credited_cents >= i64::from(gross_cents))
    {
        return InvoiceDisplayState::Cancelled;
    }
    if settlement.balance_cents == 0 {
        return InvoiceDisplayState::Paid;
    }
    if due_on.is_some_and(|due| due < today) {
        return InvoiceDisplayState::Overdue;
    }
    if settlement.paid_cents > 0 {
        InvoiceDisplayState::Partial
    } else {
        InvoiceDisplayState::Unpaid
    }
}

pub fn due_at_for(issued_on: NaiveDate, payment_terms_days: i32) -> DateTime<Utc> {
    let due = issued_on
        .checked_add_signed(Duration::days(i64::from(payment_terms_days)))
        .unwrap_or(issued_on);

    Utc.from_utc_datetime(&due.and_hms_opt(0, 0, 0).expect("midnight exists"))
}

pub fn format_euros(cents: i32) -> String {
    format!("{},{:02}\u{a0}€", cents / 100, cents % 100)
}

pub fn legal_notice_for(kind: CustomerKind, settings: &BillingSettings) -> String {
    let penalty = format!("Pénalités de retard : {}.", settings.late_penalty.as_str());

    match kind {
        CustomerKind::Individual => penalty,
        CustomerKind::Professional => format!(
            "{penalty} Indemnité forfaitaire pour frais de recouvrement : {}.",
            format_euros(settings.recovery_fee.get())
        ),
    }
}

pub fn reminder_detail(due_on: Option<NaiveDate>, today: NaiveDate) -> String {
    let late = due_on.map_or(0, |due| (today - due).num_days());

    if late > 0 {
        format!("Relance envoyée ({late} jours de retard)")
    } else {
        "Rappel envoyé".to_owned()
    }
}

pub fn validate_line(line: &InvoiceLine) -> InvoiceOutcome<()> {
    if line.label.trim().is_empty() {
        return Err(InvoiceError::invalid(
            InvoiceField::Lines,
            "a line needs a label",
        ));
    }
    if line.quantity <= Decimal::ZERO {
        return Err(InvoiceError::invalid(
            InvoiceField::Lines,
            "a line quantity must be positive",
        ));
    }
    if line
        .vat_rate_basis_points
        .is_some_and(|rate| !(0..=10_000).contains(&rate))
    {
        return Err(InvoiceError::invalid(
            InvoiceField::Lines,
            "a line vat rate must be between 0 and 10000 basis points",
        ));
    }

    Ok(())
}

pub fn discount_of(value: Decimal) -> InvoiceOutcome<DiscountPercent> {
    DiscountPercent::try_from(value).map_err(|_| {
        InvoiceError::invalid(
            InvoiceField::Lines,
            "a discount must be between 0 and 100 with at most two decimals",
        )
    })
}

pub struct IssueFacts<'a> {
    pub invoice: &'a Invoice,
    pub customer_kind: CustomerKind,
    pub registration_number: Option<&'a str>,
    pub billing_address_is_postal: bool,
    pub issue_date: NaiveDate,
    pub today: NaiveDate,
    pub last_issued_on: Option<NaiveDate>,
}

pub fn validate_issue(facts: &IssueFacts<'_>) -> InvoiceOutcome<()> {
    let invoice = facts.invoice;

    if invoice.status != InvoiceStatus::Draft {
        return Err(InvoiceError::Locked);
    }
    if invoice.lines.is_empty() {
        return Err(InvoiceError::invalid(
            InvoiceField::Lines,
            "add at least one line",
        ));
    }
    invoice.lines.iter().try_for_each(validate_line)?;
    if invoice.gross_cents <= 0 {
        return Err(InvoiceError::invalid(
            InvoiceField::Total,
            "must be greater than zero",
        ));
    }
    if !facts.billing_address_is_postal {
        return Err(InvoiceError::invalid(
            InvoiceField::BillingAddress,
            "the customer needs a billing street and city",
        ));
    }
    if facts.customer_kind == CustomerKind::Professional
        && facts
            .registration_number
            .is_none_or(|number| number.trim().is_empty())
    {
        return Err(InvoiceError::invalid(
            InvoiceField::RegistrationNumber,
            "a professional customer needs a SIRET",
        ));
    }
    if !(0..=60).contains(&invoice.payment_terms_days) {
        return Err(InvoiceError::invalid(
            InvoiceField::PaymentTermsDays,
            "must be between 0 and 60 days",
        ));
    }
    if facts.issue_date > facts.today {
        return Err(InvoiceError::DateRange {
            rule: "the issue date cannot be in the future",
        });
    }
    if facts
        .last_issued_on
        .is_some_and(|last| facts.issue_date < last)
    {
        return Err(InvoiceError::DateRange {
            rule: "the issue date cannot precede the last issued invoice",
        });
    }

    Ok(())
}

pub fn check_credit_note_date(
    today: NaiveDate,
    last_credit_note_on: Option<NaiveDate>,
    source_issued_on: Option<NaiveDate>,
) -> InvoiceOutcome<()> {
    if last_credit_note_on.is_some_and(|last| today < last) {
        return Err(InvoiceError::DateRange {
            rule: "a credit note cannot precede the last credit note",
        });
    }
    if source_issued_on.is_some_and(|issued| today < issued) {
        return Err(InvoiceError::DateRange {
            rule: "a credit note cannot precede the invoice it corrects",
        });
    }

    Ok(())
}

pub fn check_payment(
    amount_cents: i32,
    paid_on: NaiveDate,
    today: NaiveDate,
    settlement: InvoiceSettlement,
) -> InvoiceOutcome<()> {
    if amount_cents == 0 {
        return Err(InvoiceError::invalid(
            InvoiceField::AmountCents,
            "must not be zero",
        ));
    }
    if paid_on > today {
        return Err(InvoiceError::invalid(
            InvoiceField::PaidOn,
            "must not be in the future",
        ));
    }
    if amount_cents > 0 && i64::from(amount_cents) > settlement.balance_cents {
        return Err(InvoiceError::PaymentExceedsBalance);
    }
    if amount_cents < 0 && -i64::from(amount_cents) > settlement.paid_cents {
        return Err(InvoiceError::RefundExceedsCollected);
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreditableLine {
    pub line_id: InvoiceLineId,
    pub credited: Decimal,
    pub remaining: Decimal,
}

pub fn creditable_lines(source: &Invoice, credit_notes: &[Invoice]) -> Vec<CreditableLine> {
    source
        .lines
        .iter()
        .map(|line| {
            let credited: Decimal = credit_notes
                .iter()
                .flat_map(|note| note.lines.iter())
                .filter(|credit| credit.source_line_id == Some(line.id))
                .map(|credit| credit.quantity)
                .sum();

            CreditableLine {
                line_id: line.id,
                credited,
                remaining: (line.quantity - credited).max(Decimal::ZERO),
            }
        })
        .collect()
}

pub fn credit_note_lines(
    source: &Invoice,
    credit_notes: &[Invoice],
    request: &CreditRequest,
    credit_note_id: InvoiceId,
    now: DateTime<Utc>,
) -> InvoiceOutcome<Vec<InvoiceLine>> {
    let rows = creditable_lines(source, credit_notes);
    let wanted: Vec<CreditLineRequest> = match request {
        CreditRequest::Full => rows
            .iter()
            .filter(|row| row.remaining > Decimal::ZERO)
            .map(|row| CreditLineRequest {
                source_line_id: row.line_id,
                quantity: row.remaining,
            })
            .collect(),
        CreditRequest::Lines(items) => {
            let mut merged: Vec<CreditLineRequest> = Vec::new();
            for item in items.iter().filter(|item| item.quantity > Decimal::ZERO) {
                match merged
                    .iter_mut()
                    .find(|seen| seen.source_line_id == item.source_line_id)
                {
                    Some(seen) => seen.quantity += item.quantity,
                    None => merged.push(item.clone()),
                }
            }
            merged
        }
    };

    if wanted.is_empty() {
        return Err(InvoiceError::invalid(
            InvoiceField::Lines,
            "select at least one line to credit",
        ));
    }

    wanted
        .iter()
        .enumerate()
        .map(|(position, item)| {
            let row = rows
                .iter()
                .find(|row| row.line_id == item.source_line_id)
                .ok_or_else(|| {
                    InvoiceError::invalid(
                        InvoiceField::Lines,
                        "a line does not belong to the invoice",
                    )
                })?;
            if item.quantity > row.remaining {
                return Err(InvoiceError::CreditExceedsRemaining);
            }
            let line = source
                .lines
                .iter()
                .find(|line| line.id == item.source_line_id)
                .expect("the creditable row comes from the source lines");

            Ok(InvoiceLine {
                id: InvoiceLineId(generate_uuid_v7()),
                invoice_id: credit_note_id,
                source_line_id: Some(line.id),
                quantity: item.quantity,
                position: i32::try_from(position).unwrap_or(i32::MAX),
                deleted_at: None,
                created_at: now,
                updated_at: now,
                ..line.clone()
            })
        })
        .collect()
}

pub fn fully_credits(
    source: &Invoice,
    credit_notes: &[Invoice],
    credit_note_lines: &[InvoiceLine],
) -> bool {
    let rows = creditable_lines(source, credit_notes);

    rows.iter().all(|row| {
        let now_credited: Decimal = credit_note_lines
            .iter()
            .filter(|line| line.source_line_id == Some(row.line_id))
            .map(|line| line.quantity)
            .sum();

        row.remaining - now_credited <= Decimal::ZERO
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteInvoicingBlocker {
    QuoteNotAccepted,
    QuoteAlreadyInvoiced,
    DepositsCoverQuote,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteInvoicing {
    pub invoiced_percent: Decimal,
    pub remaining_percent: Decimal,
    pub can_full: bool,
    pub can_deposit: bool,
    pub blocker: Option<QuoteInvoicingBlocker>,
}

pub fn is_active_on_quote(invoice: &Invoice) -> bool {
    invoice.status == InvoiceStatus::Draft
        || invoice.derived.display_state != InvoiceDisplayState::Cancelled
}

pub fn percent_invoiced(deposit_net_cents: i64, quote_net_cents: i32) -> Decimal {
    if quote_net_cents <= 0 {
        return Decimal::ZERO;
    }

    (Decimal::from(deposit_net_cents) / Decimal::from(quote_net_cents) * Decimal::ONE_HUNDRED)
        .round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero)
}

pub fn quote_invoicing(quote: &Quote, active: &[Invoice]) -> QuoteInvoicing {
    let deposit_net: i64 = active
        .iter()
        .filter(|invoice| invoice.kind == InvoiceKind::Deposit)
        .map(|invoice| i64::from(invoice.net_cents))
        .sum();
    let invoiced_percent = percent_invoiced(deposit_net, quote.net_cents);
    let remaining_percent = (Decimal::ONE_HUNDRED - invoiced_percent).max(Decimal::ZERO);

    let blocker = if quote.status != QuoteStatus::Accepted {
        Some(QuoteInvoicingBlocker::QuoteNotAccepted)
    } else if active
        .iter()
        .any(|invoice| invoice.kind == InvoiceKind::Standard)
    {
        Some(QuoteInvoicingBlocker::QuoteAlreadyInvoiced)
    } else {
        None
    };
    let can_full = blocker.is_none();
    let can_deposit = can_full && invoiced_percent < Decimal::ONE_HUNDRED;
    let blocker = blocker.or((!can_deposit).then_some(QuoteInvoicingBlocker::DepositsCoverQuote));

    QuoteInvoicing {
        invoiced_percent,
        remaining_percent,
        can_full,
        can_deposit,
        blocker,
    }
}

pub fn format_rate(rate_bp: i32) -> String {
    (Decimal::from(rate_bp) / Decimal::ONE_HUNDRED)
        .normalize()
        .to_string()
        .replace('.', ",")
}

fn quote_net_by_rate(quote: &Quote) -> InvoiceOutcome<BTreeMap<i32, i64>> {
    let mut bases: BTreeMap<i32, i64> = BTreeMap::new();

    for line in &quote.lines {
        let base = bases.entry(line.vat_rate_bp.unwrap_or(0)).or_insert(0);
        *base = base
            .checked_add(line.net_cents().map_err(|_| out_of_bounds())?)
            .ok_or_else(out_of_bounds)?;
    }

    Ok(bases)
}

pub struct DocumentLines {
    pub organization_id: OrganizationId,
    pub invoice_id: InvoiceId,
    pub now: DateTime<Utc>,
}

impl DocumentLines {
    fn lump(
        &self,
        position: usize,
        label: String,
        unit_price_cents: i32,
        rate: i32,
    ) -> InvoiceLine {
        InvoiceLine {
            id: InvoiceLineId(generate_uuid_v7()),
            organization_id: self.organization_id,
            invoice_id: self.invoice_id,
            label,
            quantity: Decimal::ONE,
            unit_price_cents,
            vat_rate_basis_points: Some(rate),
            discount_percent: DiscountPercent::ZERO,
            product_id: None,
            service_rate_id: None,
            source_line_id: None,
            position: i32::try_from(position).unwrap_or(i32::MAX),
            deleted_at: None,
            created_at: self.now,
            updated_at: self.now,
        }
    }
}

pub fn deposit_lines(
    quote: &Quote,
    percent: DepositPercent,
    target: &DocumentLines,
) -> InvoiceOutcome<Vec<InvoiceLine>> {
    let bases = quote_net_by_rate(quote)?;
    let several = bases.len() > 1;
    let shown = percent.get().normalize();
    let reference = quote
        .reference
        .as_deref()
        .map(|reference| format!(" {reference}"))
        .unwrap_or_default();

    bases
        .into_iter()
        .enumerate()
        .map(|(position, (rate, base))| {
            let price = (Decimal::from(base) * percent.get() / Decimal::ONE_HUNDRED)
                .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
                .to_i32()
                .ok_or_else(out_of_bounds)?;
            let suffix = if several {
                format!(" (TVA {} %)", format_rate(rate))
            } else {
                String::new()
            };

            Ok(target.lump(
                position,
                format!(
                    "Acompte de {shown} % sur devis{reference}{suffix}\n{}",
                    quote.title
                ),
                price,
                rate,
            ))
        })
        .collect()
}

pub fn full_invoice_lines(
    quote: &Quote,
    issued_deposits: &[Invoice],
    target: &DocumentLines,
) -> InvoiceOutcome<Vec<InvoiceLine>> {
    let mut lines: Vec<InvoiceLine> = quote
        .lines
        .iter()
        .enumerate()
        .map(|(position, line)| InvoiceLine {
            id: InvoiceLineId(generate_uuid_v7()),
            organization_id: target.organization_id,
            invoice_id: target.invoice_id,
            label: line.label.clone(),
            quantity: line.quantity,
            unit_price_cents: line.unit_price_cents,
            vat_rate_basis_points: line.vat_rate_bp,
            discount_percent: line.discount_percent,
            product_id: line.product_id,
            service_rate_id: line.service_rate_id,
            source_line_id: None,
            position: i32::try_from(position).unwrap_or(i32::MAX),
            deleted_at: None,
            created_at: target.now,
            updated_at: target.now,
        })
        .collect();

    for deposit in issued_deposits {
        let label = format!(
            "Acompte déjà facturé ({})",
            deposit.number.as_deref().unwrap_or("brouillon")
        );
        for (rate, base) in net_by_rate(&deposit.lines)? {
            let price = i32::try_from(-base).map_err(|_| out_of_bounds())?;
            lines.push(target.lump(lines.len(), label.clone(), price, rate));
        }
    }

    Ok(lines)
}

#[cfg(test)]
mod tests;
