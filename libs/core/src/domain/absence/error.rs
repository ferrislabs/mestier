#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AbsenceError {
    #[error("end_date: must not precede the start of the absence")]
    EndBeforeStart,
    #[error("note: must not be blank")]
    NoteBlank,
    #[error("type_id: an archived absence type cannot be chosen")]
    TypeArchived,
    #[error("type_id: an absence type or a kind is required")]
    TypeRequired,
    #[error("proof_file_key: not a valid storage key")]
    ProofKeyInvalid,
    #[error("proof_file_key: this absence type needs a proof before approval")]
    ProofRequired,
    #[error("comment: a rejection needs a comment")]
    CommentRequired,
    #[error("status: the absence was already decided")]
    AlreadyDecided,
}

impl AbsenceError {
    pub fn field(self) -> &'static str {
        match self {
            Self::EndBeforeStart => "end_date",
            Self::NoteBlank => "note",
            Self::TypeArchived | Self::TypeRequired => "type_id",
            Self::ProofKeyInvalid | Self::ProofRequired => "proof_file_key",
            Self::CommentRequired => "comment",
            Self::AlreadyDecided => "status",
        }
    }
}

pub type AbsenceOutcome<T> = Result<T, AbsenceError>;
