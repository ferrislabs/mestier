#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SurchargeError {
    #[error("name: must not be empty")]
    NameEmpty,
    #[error("rate_bp: must be between 0 and 20000 basis points")]
    RateInvalid,
}

impl SurchargeError {
    pub fn field(self) -> &'static str {
        match self {
            Self::NameEmpty => "name",
            Self::RateInvalid => "rate_bp",
        }
    }
}

pub type SurchargeOutcome<T> = Result<T, SurchargeError>;
