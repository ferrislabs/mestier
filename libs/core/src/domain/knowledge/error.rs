use super::documents::ContentError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum KnowledgeError {
    #[error("title: a title cannot be empty")]
    TitleEmpty,
    #[error("title: a title holds at most 200 characters")]
    TitleTooLong,
    #[error("body: a document cannot be empty")]
    DocumentEmpty,
    #[error("body: a document holds at most 100000 characters of Markdown")]
    DocumentTooLong,
    #[error("body: only a document has a text of its own")]
    NotADocument,
    #[error("document: this document is being edited by someone else")]
    BeingEdited,
    #[error(transparent)]
    Content(ContentError),
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
    #[error("name: a folder name cannot be empty")]
    FolderNameEmpty,
    #[error("name: a folder name holds at most 100 characters")]
    FolderNameTooLong,
    #[error("name: a folder with this name already exists here")]
    FolderNameTaken,
    #[error("parent: folders nest at most 10 levels deep")]
    FolderTooDeep,
    #[error("parent: a folder cannot move inside itself")]
    FolderCycle,
    #[error("folder: only an empty folder can be deleted")]
    FolderNotEmpty,
    #[error("articles: at most 100 articles move at once")]
    TooManyArticles,
}

impl KnowledgeError {
    /// The request is well formed but clashes with what is already stored.
    pub fn is_conflict(self) -> bool {
        matches!(
            self,
            Self::FolderNameTaken | Self::FolderNotEmpty | Self::BeingEdited
        )
    }
}

pub type KnowledgeOutcome<T> = Result<T, KnowledgeError>;
