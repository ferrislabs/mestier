#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum KnowledgeError {
    #[error("title: a title cannot be empty")]
    TitleEmpty,
    #[error("title: a title holds at most 200 characters")]
    TitleTooLong,
    #[error("note: a note cannot be empty")]
    NoteEmpty,
    #[error("note: a note holds at most 100000 characters")]
    NoteTooLong,
    #[error("tags: a tag cannot be empty")]
    TagEmpty,
    #[error("tags: a tag holds at most 40 characters")]
    TagTooLong,
    #[error("tags: an article holds at most 20 tags")]
    TooManyTags,
    #[error("file: a file cannot be empty")]
    EmptyFile,
    #[error("file: a file holds at most 25 MB")]
    FileTooLarge,
    #[error("file: this file type is not accepted")]
    UnsupportedFile,
}

pub type KnowledgeOutcome<T> = Result<T, KnowledgeError>;
