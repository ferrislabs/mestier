use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub mod commands;
pub mod error;
pub mod ports;

pub use commands::*;
pub use error::{NewsError, NewsOutcome};

pub const MAX_NEWS_ITEMS: usize = 6;
pub const MAX_SEEN_IDS: usize = 50;

const TITLE_MAX: usize = 120;
const ITEM_TITLE_MAX: usize = 60;
const ITEM_TEXT_MAX: usize = 240;
const CTA_LABEL_MAX: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct NewsId(pub Uuid);

impl FromStr for NewsId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(NewsId)
    }
}

impl Display for NewsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A closed list: the webapp maps each name to one of its icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NewsIcon {
    Plug,
    Sparkles,
    Rocket,
    Chart,
    Bell,
    Calendar,
    File,
    Users,
    Shield,
    Zap,
    Image,
    Message,
    Settings,
    Map,
    Wrench,
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsItem {
    pub icon: NewsIcon,
    pub title: String,
    pub text: String,
}

impl NewsItem {
    pub fn new(icon: NewsIcon, title: &str, text: &str) -> NewsOutcome<Self> {
        let title = bounded(
            title,
            ITEM_TITLE_MAX,
            NewsError::ItemTitleEmpty,
            NewsError::ItemTitleTooLong,
        )?;
        let text = bounded(
            text,
            ITEM_TEXT_MAX,
            NewsError::ItemTextEmpty,
            NewsError::ItemTextTooLong,
        )?;
        Ok(Self { icon, title, text })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsCta {
    pub label: String,
    pub url: String,
}

impl NewsCta {
    /// Only `http` and `https` addresses are kept: the link ends up in an
    /// anchor, and a `javascript:` one would run in the reader's session.
    pub fn new(label: &str, url: &str) -> NewsOutcome<Self> {
        let label = bounded(
            label,
            CTA_LABEL_MAX,
            NewsError::CtaLabelEmpty,
            NewsError::CtaLabelTooLong,
        )?;
        let parsed = url::Url::parse(url.trim()).map_err(|_| NewsError::CtaUrlInvalid)?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err(NewsError::CtaUrlInvalid);
        }
        Ok(Self {
            label,
            url: parsed.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewsItemInput {
    pub icon: NewsIcon,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewsCtaInput {
    pub label: String,
    pub url: String,
}

/// What a caller sends, before any rule has been applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewsContentInput {
    pub title: String,
    pub image_key: Option<String>,
    pub items: Vec<NewsItemInput>,
    pub cta: Option<NewsCtaInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewsContent {
    pub title: String,
    pub image_key: Option<String>,
    pub items: Vec<NewsItem>,
    pub cta: Option<NewsCta>,
}

impl TryFrom<NewsContentInput> for NewsContent {
    type Error = NewsError;

    fn try_from(input: NewsContentInput) -> Result<Self, Self::Error> {
        let title = bounded(
            &input.title,
            TITLE_MAX,
            NewsError::TitleEmpty,
            NewsError::TitleTooLong,
        )?;
        if input.items.is_empty() {
            return Err(NewsError::NoItems);
        }
        if input.items.len() > MAX_NEWS_ITEMS {
            return Err(NewsError::TooManyItems);
        }
        let items = input
            .items
            .iter()
            .map(|item| NewsItem::new(item.icon, &item.title, &item.text))
            .collect::<NewsOutcome<Vec<_>>>()?;
        let cta = input
            .cta
            .as_ref()
            .map(|cta| NewsCta::new(&cta.label, &cta.url))
            .transpose()?;
        let image_key = input
            .image_key
            .map(|key| key.trim().to_owned())
            .filter(|key| !key.is_empty());
        Ok(Self {
            title,
            image_key,
            items,
            cta,
        })
    }
}

/// Absent `published_at` is a draft, a past one is published, a future one is
/// scheduled: there is no state field to keep in step with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct News {
    pub id: NewsId,
    pub content: NewsContent,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn check_seen_ids(ids: &[NewsId]) -> NewsOutcome<()> {
    if ids.is_empty() {
        return Err(NewsError::NoIds);
    }
    if ids.len() > MAX_SEEN_IDS {
        return Err(NewsError::TooManyIds);
    }
    Ok(())
}

fn bounded(value: &str, max: usize, empty: NewsError, too_long: NewsError) -> NewsOutcome<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(empty);
    }
    if value.chars().count() > max {
        return Err(too_long);
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(title: &str, text: &str) -> NewsItemInput {
        NewsItemInput {
            icon: NewsIcon::Sparkles,
            title: title.to_owned(),
            text: text.to_owned(),
        }
    }

    fn input() -> NewsContentInput {
        NewsContentInput {
            title: "Les nouveautés".to_owned(),
            image_key: None,
            items: vec![item("Connecteurs", "Branchez vos outils.")],
            cta: None,
        }
    }

    fn rejected(mutate: impl FnOnce(&mut NewsContentInput)) -> NewsError {
        let mut content = input();
        mutate(&mut content);
        NewsContent::try_from(content).unwrap_err()
    }

    #[test]
    fn a_valid_content_is_accepted_and_trimmed() {
        let mut content = input();
        content.title = "  Les nouveautés  ".to_owned();
        let content = NewsContent::try_from(content).unwrap();
        assert_eq!(content.title, "Les nouveautés");
        assert_eq!(content.items.len(), 1);
    }

    #[test]
    fn a_blank_title_is_refused() {
        assert_eq!(rejected(|c| c.title = "   ".into()), NewsError::TitleEmpty);
    }

    #[test]
    fn a_title_is_capped_at_120_characters() {
        let mut ok = input();
        ok.title = "é".repeat(120);
        assert!(NewsContent::try_from(ok).is_ok());
        assert_eq!(
            rejected(|c| c.title = "é".repeat(121)),
            NewsError::TitleTooLong
        );
    }

    #[test]
    fn no_item_is_refused() {
        assert_eq!(rejected(|c| c.items.clear()), NewsError::NoItems);
    }

    #[test]
    fn six_items_pass_and_seven_are_refused() {
        let mut ok = input();
        ok.items = (0..6).map(|_| item("Point", "Texte")).collect();
        assert!(NewsContent::try_from(ok).is_ok());
        assert_eq!(
            rejected(|c| c.items = (0..7).map(|_| item("Point", "Texte")).collect()),
            NewsError::TooManyItems
        );
    }

    #[test]
    fn an_item_title_is_bounded() {
        assert_eq!(
            rejected(|c| c.items = vec![item(" ", "Texte")]),
            NewsError::ItemTitleEmpty
        );
        assert_eq!(
            rejected(|c| c.items = vec![item(&"a".repeat(61), "Texte")]),
            NewsError::ItemTitleTooLong
        );
        assert!(NewsItem::new(NewsIcon::Bell, &"a".repeat(60), "Texte").is_ok());
    }

    #[test]
    fn an_item_text_is_bounded() {
        assert_eq!(
            rejected(|c| c.items = vec![item("Point", "")]),
            NewsError::ItemTextEmpty
        );
        assert_eq!(
            rejected(|c| c.items = vec![item("Point", &"a".repeat(241))]),
            NewsError::ItemTextTooLong
        );
        assert!(NewsItem::new(NewsIcon::Bell, "Point", &"a".repeat(240)).is_ok());
    }

    #[test]
    fn a_button_label_is_bounded() {
        assert_eq!(
            NewsCta::new("", "https://mestier.app").unwrap_err(),
            NewsError::CtaLabelEmpty
        );
        assert_eq!(
            NewsCta::new(&"a".repeat(41), "https://mestier.app").unwrap_err(),
            NewsError::CtaLabelTooLong
        );
        assert!(NewsCta::new(&"a".repeat(40), "https://mestier.app").is_ok());
    }

    #[test]
    fn only_http_and_https_links_are_accepted() {
        assert!(NewsCta::new("Voir", "https://mestier.app/news").is_ok());
        assert!(NewsCta::new("Voir", "http://mestier.app").is_ok());
        for url in [
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            "data:text/html,<b>x</b>",
            "ftp://mestier.app",
            "mailto:a@b.fr",
            "mestier.app",
            "",
            "https://",
        ] {
            assert_eq!(
                NewsCta::new("Voir", url).unwrap_err(),
                NewsError::CtaUrlInvalid,
                "{url}"
            );
        }
    }

    #[test]
    fn an_invalid_button_fails_the_whole_content() {
        assert_eq!(
            rejected(|c| {
                c.cta = Some(NewsCtaInput {
                    label: "Voir".into(),
                    url: "javascript:alert(1)".into(),
                })
            }),
            NewsError::CtaUrlInvalid
        );
    }

    #[test]
    fn a_blank_image_key_is_dropped() {
        let mut content = input();
        content.image_key = Some("  ".to_owned());
        assert_eq!(NewsContent::try_from(content).unwrap().image_key, None);

        let mut content = input();
        content.image_key = Some(" news/a.png ".to_owned());
        assert_eq!(
            NewsContent::try_from(content).unwrap().image_key.as_deref(),
            Some("news/a.png")
        );
    }

    #[test]
    fn the_seen_ids_are_bounded() {
        let one = NewsId(Uuid::nil());
        assert_eq!(check_seen_ids(&[]), Err(NewsError::NoIds));
        assert_eq!(check_seen_ids(&[one]), Ok(()));
        assert_eq!(check_seen_ids(&vec![one; 50]), Ok(()));
        assert_eq!(check_seen_ids(&vec![one; 51]), Err(NewsError::TooManyIds));
    }

    #[test]
    fn icons_serialize_in_snake_case() {
        assert_eq!(
            serde_json::to_string(&NewsIcon::Message).unwrap(),
            "\"message\""
        );
        assert!(serde_json::from_str::<NewsIcon>("\"unknown\"").is_err());
    }
}
