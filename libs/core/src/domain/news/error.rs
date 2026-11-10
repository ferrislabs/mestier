#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NewsError {
    #[error("title: a title cannot be empty")]
    TitleEmpty,
    #[error("title: a title holds at most 120 characters")]
    TitleTooLong,
    #[error("items: a news holds at least one item")]
    NoItems,
    #[error("items: a news holds at most 6 items")]
    TooManyItems,
    #[error("items: an item title cannot be empty")]
    ItemTitleEmpty,
    #[error("items: an item title holds at most 60 characters")]
    ItemTitleTooLong,
    #[error("items: an item text cannot be empty")]
    ItemTextEmpty,
    #[error("items: an item text holds at most 240 characters")]
    ItemTextTooLong,
    #[error("cta: a button label cannot be empty")]
    CtaLabelEmpty,
    #[error("cta: a button label holds at most 40 characters")]
    CtaLabelTooLong,
    #[error("cta: a button link must be an http or https address")]
    CtaUrlInvalid,
    #[error("ids: at least one news id is expected")]
    NoIds,
    #[error("ids: at most 50 news ids are accepted at once")]
    TooManyIds,
}

pub type NewsOutcome<T> = Result<T, NewsError>;
