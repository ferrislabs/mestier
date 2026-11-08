use super::schedule::ScheduleError;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MemberContractError {
    #[error(transparent)]
    Schedule(#[from] ScheduleError),
    #[error("end_date: must not precede the start date")]
    EndBeforeStart,
    #[error("trial_end_date: must fall during the contract")]
    TrialOutsideContract,
    #[error("end_date: this type of contract needs an end date")]
    FixedTermNeedsEndDate,
    #[error("start_date: the contract overlaps another contract of the member")]
    Overlap,
}

impl MemberContractError {
    pub fn field(&self) -> &str {
        match self {
            Self::Schedule(error) => error.field(),
            Self::EndBeforeStart | Self::FixedTermNeedsEndDate => "end_date",
            Self::TrialOutsideContract => "trial_end_date",
            Self::Overlap => "start_date",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Overlap => "E_MEMBER_CONTRACT_OVERLAP",
            _ => "E_UNPROCESSABLE_ENTITY",
        }
    }
}

pub type MemberContractOutcome<T> = Result<T, MemberContractError>;
