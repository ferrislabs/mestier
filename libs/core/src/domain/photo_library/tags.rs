use crate::domain::photo_library::error::{PhotoLibraryError, PhotoLibraryOutcome};

pub const MAX_TAGS: usize = 20;
pub const MAX_TAG_CHARS: usize = 40;

pub fn normalize_tag(raw: &str) -> PhotoLibraryOutcome<String> {
    let tag = raw.trim().to_lowercase();
    match tag.chars().count() {
        0 => Err(PhotoLibraryError::TagEmpty),
        count if count > MAX_TAG_CHARS => Err(PhotoLibraryError::TagTooLong),
        _ => Ok(tag),
    }
}

pub fn normalize_tags<S: AsRef<str>>(raw: &[S]) -> PhotoLibraryOutcome<Vec<String>> {
    let mut tags: Vec<String> = Vec::with_capacity(raw.len());
    for tag in raw {
        let tag = normalize_tag(tag.as_ref())?;
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    if tags.len() > MAX_TAGS {
        return Err(PhotoLibraryError::TooManyTags);
    }
    Ok(tags)
}

pub fn normalize_caption(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|caption| !caption.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_are_trimmed_lowercased_and_deduplicated() {
        let tags = normalize_tags(&["  Rosier ", "rosier", "TERRASSE"]).unwrap();
        assert_eq!(tags, vec!["rosier", "terrasse"]);
    }

    #[test]
    fn an_empty_tag_is_refused() {
        assert_eq!(
            normalize_tags(&["rosier", "   "]),
            Err(PhotoLibraryError::TagEmpty)
        );
    }

    #[test]
    fn a_tag_of_forty_characters_passes_and_forty_one_is_refused() {
        assert!(normalize_tags(&["é".repeat(40)]).is_ok());
        assert_eq!(
            normalize_tags(&["é".repeat(41)]),
            Err(PhotoLibraryError::TagTooLong)
        );
    }

    #[test]
    fn twenty_tags_pass_and_a_twenty_first_is_refused() {
        let twenty: Vec<String> = (0..20).map(|n| format!("tag{n}")).collect();
        assert_eq!(normalize_tags(&twenty).unwrap().len(), 20);

        let twenty_one: Vec<String> = (0..21).map(|n| format!("tag{n}")).collect();
        assert_eq!(
            normalize_tags(&twenty_one),
            Err(PhotoLibraryError::TooManyTags)
        );
    }

    #[test]
    fn duplicates_do_not_count_toward_the_limit() {
        let mut tags: Vec<String> = (0..20).map(|n| format!("tag{n}")).collect();
        tags.push("TAG0".to_owned());
        assert_eq!(normalize_tags(&tags).unwrap().len(), 20);
    }

    #[test]
    fn a_blank_caption_becomes_none() {
        assert_eq!(normalize_caption(Some("  ")), None);
        assert_eq!(
            normalize_caption(Some(" Massif ")),
            Some("Massif".to_owned())
        );
        assert_eq!(normalize_caption(None), None);
    }
}
