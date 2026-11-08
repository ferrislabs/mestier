#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LeaveAccountError {
    #[error("name: must not be empty")]
    NameEmpty,
    #[error("acquisition: must be a positive number of days with at most 2 decimals")]
    AcquisitionInvalid,
    #[error("period_start_month: must be between 1 and 12")]
    PeriodStartMonthOutOfRange,
    #[error("carry_over_max: must be zero or more days with at most 2 decimals")]
    CarryOverMaxInvalid,
}

impl LeaveAccountError {
    pub fn field(self) -> &'static str {
        match self {
            Self::NameEmpty => "name",
            Self::AcquisitionInvalid => "acquisition",
            Self::PeriodStartMonthOutOfRange => "period_start_month",
            Self::CarryOverMaxInvalid => "carry_over_max",
        }
    }
}

pub type LeaveAccountOutcome<T> = Result<T, LeaveAccountError>;
