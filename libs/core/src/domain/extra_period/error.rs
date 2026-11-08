use crate::{DayRefusal, SurchargeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ExtraPeriodError {
    #[error(transparent)]
    Day(#[from] DayRefusal),
    #[error("reason: must not be blank")]
    ReasonBlank,
    #[error("rate_bp: must be between 0 and 20000 basis points")]
    RateInvalid,
    #[error("member_id: no such member in the organization")]
    MemberUnknown,
    #[error("surcharge_id: no such surcharge in the organization")]
    SurchargeUnknown,
    #[error("surcharge_id: an archived surcharge cannot be chosen")]
    SurchargeArchived,
    #[error("comment: a rejection needs a comment")]
    CommentRequired,
    #[error("status: the extra period was already decided")]
    AlreadyDecided,
}

impl ExtraPeriodError {
    pub fn field(self) -> &'static str {
        match self {
            Self::Day(DayRefusal::TimeFormat) => "start",
            Self::Day(DayRefusal::EndBeforeStart) => "end",
            Self::Day(DayRefusal::LunchEndBeforeStart | DayRefusal::LunchOutsideDay) => "lunch",
            Self::ReasonBlank => "reason",
            Self::RateInvalid => "rate_bp",
            Self::MemberUnknown => "member_id",
            Self::SurchargeUnknown | Self::SurchargeArchived => "surcharge_id",
            Self::CommentRequired => "comment",
            Self::AlreadyDecided => "status",
        }
    }
}

impl From<SurchargeError> for ExtraPeriodError {
    fn from(_: SurchargeError) -> Self {
        Self::RateInvalid
    }
}

pub type ExtraPeriodOutcome<T> = Result<T, ExtraPeriodError>;
