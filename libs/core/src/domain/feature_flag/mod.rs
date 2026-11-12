use chrono::{DateTime, Utc};

use crate::UserId;

pub mod events;
pub mod ports;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureFlag {
    DriveAi,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown feature flag `{0}`")]
pub struct UnknownFeatureFlag(pub String);

impl FeatureFlag {
    pub const ALL: &'static [FeatureFlag] = &[FeatureFlag::DriveAi];

    pub fn key(self) -> &'static str {
        match self {
            Self::DriveAi => "drive_ai",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DriveAi => "Drive : assistant IA",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::DriveAi => "Assistant IA du Drive (résumés, recherche en langage naturel).",
        }
    }
}

impl TryFrom<&str> for FeatureFlag {
    type Error = UnknownFeatureFlag;

    fn try_from(key: &str) -> Result<Self, Self::Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|flag| flag.key() == key)
            .ok_or_else(|| UnknownFeatureFlag(key.to_owned()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureActivation {
    pub flag: FeatureFlag,
    pub enabled_at: DateTime<Utc>,
    pub enabled_by: Option<UserId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_key_resolves_to_its_flag() {
        assert_eq!(FeatureFlag::try_from("drive_ai"), Ok(FeatureFlag::DriveAi));
    }

    #[test]
    fn an_unknown_key_is_refused() {
        assert_eq!(
            FeatureFlag::try_from("inconnu"),
            Err(UnknownFeatureFlag("inconnu".to_owned()))
        );
    }

    #[test]
    fn every_flag_round_trips_through_its_key() {
        for flag in FeatureFlag::ALL {
            assert_eq!(FeatureFlag::try_from(flag.key()), Ok(*flag));
        }
    }

    #[test]
    fn keys_are_unique_snake_case_and_texts_are_filled() {
        let mut keys: Vec<&str> = FeatureFlag::ALL.iter().map(|flag| flag.key()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), FeatureFlag::ALL.len());
        for flag in FeatureFlag::ALL {
            assert!(
                flag.key()
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_')
            );
            assert!(!flag.label().is_empty());
            assert!(!flag.description().is_empty());
        }
    }

    #[test]
    fn the_drive_ai_flag_keeps_its_published_texts() {
        assert_eq!(FeatureFlag::DriveAi.label(), "Drive : assistant IA");
        assert_eq!(
            FeatureFlag::DriveAi.description(),
            "Assistant IA du Drive (résumés, recherche en langage naturel)."
        );
    }
}
