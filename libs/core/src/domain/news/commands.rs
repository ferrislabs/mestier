use chrono::{DateTime, Utc};

use crate::{NewsContentInput, NewsId, PlatformAdmin};

#[derive(Debug, Clone)]
pub struct CreateNewsCommand {
    pub admin: PlatformAdmin,
    pub content: NewsContentInput,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct UpdateNewsCommand {
    pub admin: PlatformAdmin,
    pub id: NewsId,
    pub content: NewsContentInput,
    pub published_at: Option<DateTime<Utc>>,
}
