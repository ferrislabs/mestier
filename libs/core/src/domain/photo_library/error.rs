#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PhotoLibraryError {
    #[error("image: the file is empty")]
    EmptyImage,
    #[error("image: only JPEG, PNG and WebP images are accepted")]
    UnsupportedImage,
    #[error("image: the file could not be decoded")]
    CorruptImage,
    #[error("tags: a tag cannot be empty")]
    TagEmpty,
    #[error("tags: a tag holds at most 40 characters")]
    TagTooLong,
    #[error("tags: a photo holds at most 20 tags")]
    TooManyTags,
    #[error("placements: at most 50 photos can be placed at once")]
    TooManyPlacements,
}

pub type PhotoLibraryOutcome<T> = Result<T, PhotoLibraryError>;
