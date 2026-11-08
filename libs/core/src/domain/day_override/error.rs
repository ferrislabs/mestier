use crate::DayRefusal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PlanningPeriodError {
    #[error("to: must not precede from")]
    Inverted,
    #[error("to: the period spans at most {0} days")]
    TooLong(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DayOverrideError {
    #[error("start: a working day needs a start time")]
    StartMissing,
    #[error("end: a working day needs an end time")]
    EndMissing,
    #[error("kind: a day off has no hours")]
    OffHasHours,
    #[error(transparent)]
    Day(#[from] DayRefusal),
    #[error(transparent)]
    Period(#[from] PlanningPeriodError),
}

impl DayOverrideError {
    pub fn field(self) -> &'static str {
        match self {
            Self::StartMissing | Self::Day(DayRefusal::TimeFormat) => "start",
            Self::EndMissing | Self::Day(DayRefusal::EndBeforeStart) => "end",
            Self::OffHasHours => "kind",
            Self::Day(DayRefusal::LunchEndBeforeStart | DayRefusal::LunchOutsideDay) => "lunch",
            Self::Period(_) => "to",
        }
    }
}

pub type DayOverrideOutcome<T> = Result<T, DayOverrideError>;
