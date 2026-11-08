#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LeaveAdjustmentError {
    #[error("days: must be a non-zero number of days with at most 2 decimals")]
    DaysInvalid,
    #[error("reason: must not be empty")]
    ReasonEmpty,
    #[error("days: a carry-over is always positive")]
    CarryOverNegative,
    #[error("days: the carry-over exceeds the maximum of the account")]
    CarryOverTooLarge,
    #[error("account_id: no such leave account in the organization")]
    AccountUnknown,
    #[error("account_id: the leave account is archived")]
    AccountArchived,
    #[error("member_id: no such member in the organization")]
    MemberUnknown,
}

impl LeaveAdjustmentError {
    pub fn field(self) -> &'static str {
        match self {
            Self::DaysInvalid | Self::CarryOverNegative | Self::CarryOverTooLarge => "days",
            Self::ReasonEmpty => "reason",
            Self::AccountUnknown | Self::AccountArchived => "account_id",
            Self::MemberUnknown => "member_id",
        }
    }
}

pub type LeaveAdjustmentOutcome<T> = Result<T, LeaveAdjustmentError>;
