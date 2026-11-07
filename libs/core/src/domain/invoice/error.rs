use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvoiceField {
    Title,
    Lines,
    PaymentTermsDays,
    LegalNotice,
    IssueDate,
    BillingAddress,
    RegistrationNumber,
    Total,
    Status,
    AmountCents,
    PaidOn,
    Reason,
    Percent,
    Quote,
    Note,
}

impl InvoiceField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Lines => "lines",
            Self::PaymentTermsDays => "payment_terms_days",
            Self::LegalNotice => "legal_notice",
            Self::IssueDate => "issue_date",
            Self::BillingAddress => "billing_address",
            Self::RegistrationNumber => "registration_number",
            Self::Total => "total",
            Self::Status => "status",
            Self::AmountCents => "amount_cents",
            Self::PaidOn => "paid_on",
            Self::Reason => "reason",
            Self::Percent => "percent",
            Self::Quote => "quote",
            Self::Note => "note",
        }
    }
}

impl fmt::Display for InvoiceField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvoiceError {
    #[error("{field}: {rule}")]
    Invalid {
        field: InvoiceField,
        rule: &'static str,
    },
    #[error("status: an invoice that is not a draft cannot be changed")]
    Locked,
    #[error("issue_date: {rule}")]
    DateRange { rule: &'static str },
    #[error("amount_cents: the payment exceeds the balance")]
    PaymentExceedsBalance,
    #[error("amount_cents: the refund exceeds the net amount collected")]
    RefundExceedsCollected,
    #[error("lines: a line exceeds what remains to credit on its source line")]
    CreditExceedsRemaining,
    #[error("percent: the deposit exceeds the percentage still to invoice")]
    DepositExceedsRemaining,
    #[error("quote: the quote already has an active invoice")]
    QuoteAlreadyInvoiced,
    #[error("quote: only an accepted quote can be invoiced")]
    QuoteNotAccepted,
    #[error("status: only an issued invoice accepts this")]
    NotIssued,
}

impl InvoiceError {
    pub fn invalid(field: InvoiceField, rule: &'static str) -> Self {
        Self::Invalid { field, rule }
    }

    pub fn field(self) -> InvoiceField {
        match self {
            Self::Invalid { field, .. } => field,
            Self::Locked | Self::NotIssued => InvoiceField::Status,
            Self::DateRange { .. } => InvoiceField::IssueDate,
            Self::PaymentExceedsBalance | Self::RefundExceedsCollected => InvoiceField::AmountCents,
            Self::CreditExceedsRemaining => InvoiceField::Lines,
            Self::DepositExceedsRemaining => InvoiceField::Percent,
            Self::QuoteAlreadyInvoiced | Self::QuoteNotAccepted => InvoiceField::Quote,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Invalid { .. } => "E_UNPROCESSABLE_ENTITY",
            Self::Locked => "E_INVOICE_LOCKED",
            Self::DateRange { .. } => "E_INVOICE_DATE_RANGE",
            Self::PaymentExceedsBalance => "E_PAYMENT_EXCEEDS_BALANCE",
            Self::RefundExceedsCollected => "E_REFUND_EXCEEDS_COLLECTED",
            Self::CreditExceedsRemaining => "E_CREDIT_EXCEEDS_REMAINING",
            Self::DepositExceedsRemaining => "E_DEPOSIT_EXCEEDS_REMAINING",
            Self::QuoteAlreadyInvoiced => "E_QUOTE_ALREADY_INVOICED",
            Self::QuoteNotAccepted | Self::NotIssued => "E_CONFLICT",
        }
    }

    pub fn is_state_conflict(self) -> bool {
        matches!(
            self,
            Self::Locked | Self::QuoteAlreadyInvoiced | Self::QuoteNotAccepted | Self::NotIssued
        )
    }
}

pub type InvoiceOutcome<T> = Result<T, InvoiceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_refusal_carries_its_frozen_code_and_field() {
        let cases = [
            (InvoiceError::Locked, "E_INVOICE_LOCKED", "status", true),
            (
                InvoiceError::DateRange { rule: "x" },
                "E_INVOICE_DATE_RANGE",
                "issue_date",
                false,
            ),
            (
                InvoiceError::PaymentExceedsBalance,
                "E_PAYMENT_EXCEEDS_BALANCE",
                "amount_cents",
                false,
            ),
            (
                InvoiceError::RefundExceedsCollected,
                "E_REFUND_EXCEEDS_COLLECTED",
                "amount_cents",
                false,
            ),
            (
                InvoiceError::CreditExceedsRemaining,
                "E_CREDIT_EXCEEDS_REMAINING",
                "lines",
                false,
            ),
            (
                InvoiceError::DepositExceedsRemaining,
                "E_DEPOSIT_EXCEEDS_REMAINING",
                "percent",
                false,
            ),
            (
                InvoiceError::QuoteAlreadyInvoiced,
                "E_QUOTE_ALREADY_INVOICED",
                "quote",
                true,
            ),
        ];

        for (error, code, field, conflict) in cases {
            assert_eq!(error.code(), code);
            assert_eq!(error.field().as_str(), field);
            assert_eq!(error.is_state_conflict(), conflict);
        }
    }

    #[test]
    fn a_plain_rule_answers_unprocessable_on_its_field() {
        let error = InvoiceError::invalid(InvoiceField::Reason, "must not be blank");

        assert_eq!(error.code(), "E_UNPROCESSABLE_ENTITY");
        assert_eq!(error.field().as_str(), "reason");
        assert!(!error.is_state_conflict());
    }
}
