use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{News, NewsContent, NewsId};

#[derive(Debug, Clone)]
pub struct NewsRow {
    pub id: Uuid,
    pub title: String,
    pub image_key: Option<String>,
    pub items: serde_json::Value,
    pub cta: Option<serde_json::Value>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<NewsRow> for News {
    type Error = CoreError;

    fn try_from(row: NewsRow) -> Result<Self, Self::Error> {
        let items = serde_json::from_value(row.items)
            .map_err(|e| CoreError::Internal(format!("malformed news items: {e}")))?;
        let cta = row
            .cta
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| CoreError::Internal(format!("malformed news cta: {e}")))?;
        Ok(Self {
            id: NewsId(row.id),
            content: NewsContent {
                title: row.title,
                image_key: row.image_key,
                items,
                cta,
            },
            published_at: row.published_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
