use authz::Subject;
use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, UserId};

pub mod events;
pub mod ports;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureFlag {
    DriveAi,
    Surveys,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown feature flag `{0}`")]
pub struct UnknownFeatureFlag(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("feature flag `{}` is not self-service", .0.key())]
pub struct NotSelfServiceFeature(pub FeatureFlag);

impl From<NotSelfServiceFeature> for CoreError {
    fn from(_: NotSelfServiceFeature) -> Self {
        CoreError::NotFound
    }
}

impl FeatureFlag {
    pub const ALL: &'static [FeatureFlag] = &[FeatureFlag::DriveAi, FeatureFlag::Surveys];

    pub fn key(self) -> &'static str {
        match self {
            Self::DriveAi => "drive_ai",
            Self::Surveys => "surveys",
        }
    }

    pub fn is_self_service(self) -> bool {
        match self {
            Self::DriveAi => false,
            Self::Surveys => true,
        }
    }

    pub fn require_self_service(self) -> Result<Self, NotSelfServiceFeature> {
        if self.is_self_service() {
            Ok(self)
        } else {
            Err(NotSelfServiceFeature(self))
        }
    }

    pub fn self_service_states(enabled: &[FeatureFlag]) -> Vec<(FeatureFlag, bool)> {
        Self::ALL
            .iter()
            .copied()
            .filter(|flag| flag.is_self_service())
            .map(|flag| (flag, enabled.contains(&flag)))
            .collect()
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DriveAi => "Drive : assistant IA",
            Self::Surveys => "Questionnaires",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::DriveAi => "Assistant IA du Drive (résumés, recherche en langage naturel).",
            Self::Surveys => {
                "Questionnaires et campagnes de réponses, ouverts au public ou aux membres."
            }
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

pub struct SetSelfServiceFeatureCommand {
    pub actor: Subject,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
    pub flag: FeatureFlag,
    pub enabled: bool,
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
    fn surveys_is_self_service_and_drive_ai_stays_staff_only() {
        assert!(FeatureFlag::Surveys.is_self_service());
        assert!(!FeatureFlag::DriveAi.is_self_service());
    }

    #[test]
    fn a_self_service_flag_passes_the_check_and_a_staff_flag_is_refused() {
        assert_eq!(
            FeatureFlag::Surveys.require_self_service(),
            Ok(FeatureFlag::Surveys)
        );
        assert_eq!(
            FeatureFlag::DriveAi.require_self_service(),
            Err(NotSelfServiceFeature(FeatureFlag::DriveAi))
        );
    }

    #[test]
    fn a_staff_only_flag_is_not_found_for_an_organization() {
        assert!(matches!(
            CoreError::from(NotSelfServiceFeature(FeatureFlag::DriveAi)),
            CoreError::NotFound
        ));
    }

    #[test]
    fn the_self_service_states_list_only_self_service_flags_with_their_state() {
        assert_eq!(
            FeatureFlag::self_service_states(&[FeatureFlag::DriveAi]),
            vec![(FeatureFlag::Surveys, false)]
        );
        assert_eq!(
            FeatureFlag::self_service_states(&[FeatureFlag::Surveys]),
            vec![(FeatureFlag::Surveys, true)]
        );
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
