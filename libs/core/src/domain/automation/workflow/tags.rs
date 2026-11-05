use std::collections::HashSet;

pub const MAX_TAGS: usize = 10;
pub const MAX_TAG_LENGTH: usize = 30;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowTags(Vec<String>);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WorkflowTagsError {
    #[error("a workflow carries at most {MAX_TAGS} tags")]
    TooMany,
    #[error("a tag cannot be empty")]
    Empty,
    #[error("the tag `{tag}` holds more than {MAX_TAG_LENGTH} characters")]
    TooLong { tag: String },
}

impl WorkflowTags {
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_vec(self) -> Vec<String> {
        self.0
    }
}

impl TryFrom<Vec<String>> for WorkflowTags {
    type Error = WorkflowTagsError;

    fn try_from(raw: Vec<String>) -> Result<Self, Self::Error> {
        let mut seen = HashSet::with_capacity(raw.len());
        let mut kept = Vec::with_capacity(raw.len().min(MAX_TAGS));

        for tag in raw {
            let trimmed = tag.trim();
            if trimmed.is_empty() {
                return Err(WorkflowTagsError::Empty);
            }
            if trimmed.chars().count() > MAX_TAG_LENGTH {
                return Err(WorkflowTagsError::TooLong {
                    tag: trimmed.to_owned(),
                });
            }
            if seen.insert(trimmed.to_lowercase()) {
                kept.push(trimmed.to_owned());
            }
        }

        if kept.len() > MAX_TAGS {
            return Err(WorkflowTagsError::TooMany);
        }

        Ok(Self(kept))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(raw: &[&str]) -> Result<WorkflowTags, WorkflowTagsError> {
        WorkflowTags::try_from(raw.iter().map(|tag| (*tag).to_owned()).collect::<Vec<_>>())
    }

    #[test]
    fn no_tag_is_valid() {
        assert!(tags(&[]).expect("empty list").is_empty());
    }

    #[test]
    fn tags_are_trimmed_and_keep_their_order() {
        let parsed = tags(&["  facturation ", "crm", " relance"]).expect("valid");

        assert_eq!(parsed.as_slice(), ["facturation", "crm", "relance"]);
    }

    #[test]
    fn duplicates_ignoring_case_collapse_onto_the_first_spelling() {
        let parsed = tags(&["CRM", "relance", "crm", " Relance "]).expect("valid");

        assert_eq!(parsed.as_slice(), ["CRM", "relance"]);
    }

    #[test]
    fn ten_distinct_tags_are_accepted_and_eleven_are_refused() {
        let ten: Vec<String> = (0..10).map(|index| format!("tag-{index}")).collect();
        let eleven: Vec<String> = (0..11).map(|index| format!("tag-{index}")).collect();

        assert!(WorkflowTags::try_from(ten).is_ok());
        assert_eq!(
            WorkflowTags::try_from(eleven),
            Err(WorkflowTagsError::TooMany)
        );
    }

    #[test]
    fn the_limit_counts_tags_after_deduplication() {
        let mut raw: Vec<String> = (0..10).map(|index| format!("tag-{index}")).collect();
        raw.push("TAG-0".to_owned());

        let parsed = WorkflowTags::try_from(raw).expect("ten distinct tags");

        assert_eq!(parsed.as_slice().len(), 10);
    }

    #[test]
    fn an_empty_or_blank_tag_is_refused() {
        assert_eq!(tags(&[""]), Err(WorkflowTagsError::Empty));
        assert_eq!(tags(&["crm", "   "]), Err(WorkflowTagsError::Empty));
    }

    #[test]
    fn thirty_characters_pass_and_thirty_one_do_not() {
        let thirty = "é".repeat(30);
        let thirty_one = "é".repeat(31);

        assert!(tags(&[&thirty]).is_ok());
        assert_eq!(
            tags(&[&thirty_one]),
            Err(WorkflowTagsError::TooLong { tag: thirty_one })
        );
    }

    #[test]
    fn the_length_limit_applies_after_trimming() {
        let padded = format!("  {}  ", "a".repeat(30));

        assert!(tags(&[&padded]).is_ok());
    }
}
