#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AbsenceTypeError {
    #[error("name: must not be empty")]
    NameEmpty,
    #[error("hue: must be between 0 and 360")]
    HueOutOfRange,
}

impl AbsenceTypeError {
    pub fn field(self) -> &'static str {
        match self {
            Self::NameEmpty => "name",
            Self::HueOutOfRange => "hue",
        }
    }
}

pub type AbsenceTypeOutcome<T> = Result<T, AbsenceTypeError>;
