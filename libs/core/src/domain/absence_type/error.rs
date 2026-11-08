#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AbsenceTypeError {
    #[error("name: must not be empty")]
    NameEmpty,
    #[error("hue: must be between 0 and 360")]
    HueOutOfRange,
    #[error("account_id: no such leave account in the organization")]
    AccountUnknown,
    #[error("account_id: the leave account is archived")]
    AccountArchived,
}

impl AbsenceTypeError {
    pub fn field(self) -> &'static str {
        match self {
            Self::NameEmpty => "name",
            Self::HueOutOfRange => "hue",
            Self::AccountUnknown | Self::AccountArchived => "account_id",
        }
    }
}

pub type AbsenceTypeOutcome<T> = Result<T, AbsenceTypeError>;
