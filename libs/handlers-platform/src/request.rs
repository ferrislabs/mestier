use chrono::{DateTime, Utc};
use mestier_core::{NewsContentInput, NewsCtaInput, NewsIcon, NewsItemInput};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsItemRequest {
    pub icon: NewsIcon,
    /// 1 to 60 characters.
    pub title: String,
    /// 1 to 240 characters.
    pub text: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsCtaRequest {
    /// 1 to 40 characters.
    pub label: String,
    /// An `http://` or `https://` address.
    pub url: String,
}

/// Body of both the creation and the update: an update replaces the whole news.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewsRequest {
    /// 1 to 120 characters.
    pub title: String,
    /// Key of an uploaded image. Blank is the same as absent.
    #[serde(default)]
    pub image_key: Option<String>,
    /// 1 to 6 items.
    pub items: Vec<NewsItemRequest>,
    #[serde(default)]
    pub cta: Option<NewsCtaRequest>,
    /// Absent is a draft, a past date is published, a future date is scheduled.
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
}

impl NewsRequest {
    pub fn into_parts(self) -> (NewsContentInput, Option<DateTime<Utc>>) {
        let content = NewsContentInput {
            title: self.title,
            image_key: self.image_key,
            items: self
                .items
                .into_iter()
                .map(|item| NewsItemInput {
                    icon: item.icon,
                    title: item.title,
                    text: item.text,
                })
                .collect(),
            cta: self.cta.map(|cta| NewsCtaInput {
                label: cta.label,
                url: cta.url,
            }),
        };
        (content, self.published_at)
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkSeenRequest {
    /// 1 to 50 news ids. Unknown or unpublished ones are ignored.
    pub ids: Vec<mestier_core::NewsId>,
}
