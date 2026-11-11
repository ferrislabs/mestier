use crate::domain::knowledge::tables::TableError;

pub const MAX_NAME_CHARS: usize = 100;
pub const MAX_DESCRIPTION_CHARS: usize = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableItemName(String);

impl TableItemName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn same_as(&self, other: &str) -> bool {
        self.0.to_lowercase() == other.trim().to_lowercase()
    }
}

impl TryFrom<&str> for TableItemName {
    type Error = TableError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(TableError::NameEmpty);
        }
        if trimmed.chars().count() > MAX_NAME_CHARS {
            return Err(TableError::NameTooLong);
        }
        Ok(Self(trimmed.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDescription(String);

impl TableDescription {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(raw: Option<&str>) -> Result<Option<Self>, TableError> {
        let Some(trimmed) = raw.map(str::trim).filter(|text| !text.is_empty()) else {
            return Ok(None);
        };
        if trimmed.chars().count() > MAX_DESCRIPTION_CHARS {
            return Err(TableError::DescriptionTooLong);
        }
        Ok(Some(Self(trimmed.to_owned())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_trimmed_and_bounded() {
        assert_eq!(
            TableItemName::try_from("  Chantiers ").unwrap().as_str(),
            "Chantiers"
        );
        assert_eq!(TableItemName::try_from("   "), Err(TableError::NameEmpty));
        assert_eq!(
            TableItemName::try_from("x".repeat(101).as_str()),
            Err(TableError::NameTooLong)
        );
        assert!(
            TableItemName::try_from("Statut")
                .unwrap()
                .same_as(" statut")
        );
    }

    #[test]
    fn a_blank_description_is_none() {
        assert_eq!(TableDescription::parse(Some("  ")), Ok(None));
        assert_eq!(
            TableDescription::parse(Some(&"x".repeat(2_001))),
            Err(TableError::DescriptionTooLong)
        );
    }
}
