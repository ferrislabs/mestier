use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DealField {
    Title,
    AmountCents,
    OwnerMemberId,
    LostReason,
    Quotes,
}

impl DealField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::AmountCents => "amount_cents",
            Self::OwnerMemberId => "owner_member_id",
            Self::LostReason => "lost_reason",
            Self::Quotes => "quotes",
        }
    }
}

impl fmt::Display for DealField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DealError {
    #[error("title: must not be empty")]
    TitleRequired,
    #[error("amount_cents: must not be negative")]
    AmountNegative,
    #[error("owner_member_id: must be a member of the organization")]
    OwnerNotMember,
    #[error("lost_reason: a lost deal needs a reason")]
    LostReasonRequired,
    #[error("quotes: a deal linked to a quote cannot be deleted")]
    HasQuote,
}

impl DealError {
    pub fn field(self) -> DealField {
        match self {
            Self::TitleRequired => DealField::Title,
            Self::AmountNegative => DealField::AmountCents,
            Self::OwnerNotMember => DealField::OwnerMemberId,
            Self::LostReasonRequired => DealField::LostReason,
            Self::HasQuote => DealField::Quotes,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::LostReasonRequired => "E_DEAL_LOST_REASON_REQUIRED",
            Self::HasQuote => "E_DEAL_HAS_QUOTE",
            Self::TitleRequired | Self::AmountNegative | Self::OwnerNotMember => {
                "E_UNPROCESSABLE_ENTITY"
            }
        }
    }
}

pub type DealOutcome<T> = Result<T, DealError>;
