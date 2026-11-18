use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::error::SurveyError;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "BTreeMap<String, String>")]
pub struct Metadata(BTreeMap<String, String>);

impl Metadata {
    pub fn new(entries: BTreeMap<String, String>) -> Result<Self, SurveyError> {
        for (key, value) in &entries {
            if key.trim().is_empty() {
                return Err(SurveyError::MetadataEmptyKey);
            }
            if value.trim().is_empty() {
                return Err(SurveyError::MetadataEmptyValue);
            }
        }
        Ok(Self(entries))
    }

    pub fn merged(campaign: &Metadata, survey: &Metadata) -> Metadata {
        let mut entries = campaign.0.clone();
        entries.extend(survey.0.iter().map(|(k, v)| (k.clone(), v.clone())));
        Metadata(entries)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl TryFrom<BTreeMap<String, String>> for Metadata {
    type Error = SurveyError;

    fn try_from(entries: BTreeMap<String, String>) -> Result<Self, Self::Error> {
        Self::new(entries)
    }
}
