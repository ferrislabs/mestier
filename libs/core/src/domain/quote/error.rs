use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuoteField {
    Title,
    Lines,
    CustomerId,
    OwnerMemberId,
    BillingContactId,
    BillingAddressId,
    ValidUntil,
    DepositPercent,
    DiscountPercent,
    BillingFrequency,
    CatalogueReference,
    Status,
    Total,
    BillingAddress,
    Notes,
}

impl QuoteField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Lines => "lines",
            Self::CustomerId => "customer_id",
            Self::OwnerMemberId => "owner_member_id",
            Self::BillingContactId => "billing_contact_id",
            Self::BillingAddressId => "billing_address_id",
            Self::ValidUntil => "valid_until",
            Self::DepositPercent => "deposit_percent",
            Self::DiscountPercent => "discount_percent",
            Self::BillingFrequency => "billing_frequency",
            Self::CatalogueReference => "catalogue_reference",
            Self::Status => "status",
            Self::Total => "total",
            Self::BillingAddress => "billing_address",
            Self::Notes => "notes",
        }
    }
}

impl fmt::Display for QuoteField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum QuoteError {
    #[error("{field}: {rule}")]
    Invalid {
        field: QuoteField,
        rule: &'static str,
    },
    #[error("status: a quote that is not a draft cannot be changed")]
    Locked,
    #[error("valid_until: the quote has expired")]
    Expired,
    #[error("status: only a sent quote can be answered")]
    NotSent,
    #[error("status: only a sent or declined quote can be revised")]
    NotRevisable,
}

impl QuoteError {
    pub fn invalid(field: QuoteField, rule: &'static str) -> Self {
        Self::Invalid { field, rule }
    }

    pub fn field(self) -> QuoteField {
        match self {
            Self::Invalid { field, .. } => field,
            Self::Expired => QuoteField::ValidUntil,
            Self::Locked | Self::NotSent | Self::NotRevisable => QuoteField::Status,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Locked => "E_QUOTE_LOCKED",
            Self::Expired => "E_QUOTE_EXPIRED",
            Self::NotSent => "E_QUOTE_NOT_SENT",
            Self::NotRevisable => "E_CONFLICT",
            Self::Invalid { .. } => "E_UNPROCESSABLE_ENTITY",
        }
    }

    pub fn is_state_conflict(self) -> bool {
        !matches!(self, Self::Invalid { .. })
    }
}

pub type QuoteOutcome<T> = Result<T, QuoteError>;
