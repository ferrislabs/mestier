use chrono::{DateTime, Utc};
use mestier_core::{News, NewsIcon, NewsId};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct NewsItemResponse {
    pub icon: NewsIcon,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct NewsCtaResponse {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct NewsResponse {
    pub id: NewsId,
    pub title: String,
    pub image_key: Option<String>,
    pub items: Vec<NewsItemResponse>,
    pub cta: Option<NewsCtaResponse>,
    /// Absent for a draft, in the future for a scheduled news.
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<News> for NewsResponse {
    fn from(news: News) -> Self {
        let content = news.content;
        Self {
            id: news.id,
            title: content.title,
            image_key: content.image_key,
            items: content
                .items
                .into_iter()
                .map(|item| NewsItemResponse {
                    icon: item.icon,
                    title: item.title,
                    text: item.text,
                })
                .collect(),
            cta: content.cta.map(|cta| NewsCtaResponse {
                label: cta.label,
                url: cta.url,
            }),
            published_at: news.published_at,
            created_at: news.created_at,
            updated_at: news.updated_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct NewsAccessResponse {
    pub can_manage: bool,
}
