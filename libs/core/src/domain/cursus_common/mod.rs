use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ArchivedFilter {
    #[default]
    Active,
    Archived,
    All,
}

impl ArchivedFilter {
    pub fn archived(self) -> Option<bool> {
        match self {
            Self::Active => Some(false),
            Self::Archived => Some(true),
            Self::All => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPattern(String);

impl SearchPattern {
    pub fn parse(raw: Option<&str>) -> Option<Self> {
        let trimmed = raw?.trim();
        if trimmed.is_empty() {
            return None;
        }
        let escaped = trimmed
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        Some(Self(format!("%{escaped}%")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern(raw: &str) -> Option<String> {
        SearchPattern::parse(Some(raw)).map(|pattern| pattern.as_str().to_owned())
    }

    #[test]
    fn absent_query_is_no_filter() {
        assert_eq!(SearchPattern::parse(None), None);
    }

    #[test]
    fn blank_query_is_no_filter() {
        assert_eq!(pattern(""), None);
        assert_eq!(pattern("   \t "), None);
    }

    #[test]
    fn query_is_trimmed_and_wrapped_as_a_substring() {
        assert_eq!(pattern("  dupont "), Some("%dupont%".to_owned()));
    }

    #[test]
    fn inner_spaces_are_kept() {
        assert_eq!(pattern("jean dup"), Some("%jean dup%".to_owned()));
    }

    #[test]
    fn like_wildcards_are_escaped() {
        assert_eq!(pattern("50%"), Some("%50\\%%".to_owned()));
        assert_eq!(pattern("a_b"), Some("%a\\_b%".to_owned()));
    }

    #[test]
    fn backslash_is_escaped_before_wildcards() {
        assert_eq!(pattern("a\\%"), Some("%a\\\\\\%%".to_owned()));
    }

    #[test]
    fn archived_filter_defaults_to_active() {
        assert_eq!(ArchivedFilter::default(), ArchivedFilter::Active);
    }

    #[test]
    fn archived_filter_maps_to_the_archived_state_it_keeps() {
        assert_eq!(ArchivedFilter::Active.archived(), Some(false));
        assert_eq!(ArchivedFilter::Archived.archived(), Some(true));
        assert_eq!(ArchivedFilter::All.archived(), None);
    }

    #[test]
    fn archived_filter_is_lowercase_on_the_wire() {
        let parsed: Vec<ArchivedFilter> =
            serde_json::from_str(r#"["active","archived","all"]"#).unwrap();
        assert_eq!(
            parsed,
            vec![
                ArchivedFilter::Active,
                ArchivedFilter::Archived,
                ArchivedFilter::All
            ]
        );
        assert!(serde_json::from_str::<ArchivedFilter>(r#""true""#).is_err());
    }
}
