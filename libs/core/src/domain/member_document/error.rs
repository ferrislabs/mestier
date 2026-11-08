#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MemberDocumentError {
    #[error("name: must not be empty")]
    NameEmpty,
    #[error("file_key: not a valid storage key")]
    FileKeyInvalid,
}

impl MemberDocumentError {
    pub fn field(self) -> &'static str {
        match self {
            Self::NameEmpty => "name",
            Self::FileKeyInvalid => "file_key",
        }
    }
}

pub type MemberDocumentOutcome<T> = Result<T, MemberDocumentError>;
