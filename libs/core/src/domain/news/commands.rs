use chrono::{DateTime, Utc};

use crate::{NewsContentInput, NewsEditor, NewsId};

#[derive(Debug, Clone)]
pub struct CreateNewsCommand {
    pub editor: NewsEditor,
    pub content: NewsContentInput,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct UpdateNewsCommand {
    pub editor: NewsEditor,
    pub id: NewsId,
    pub content: NewsContentInput,
    pub published_at: Option<DateTime<Utc>>,
}
