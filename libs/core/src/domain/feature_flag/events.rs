use events::{DomainEvent, EventSubject};
use serde_json::{Value, json};

use crate::{FeatureFlag, OrganizationId};

pub const FEATURE_ENABLED: &str = "organization.feature_enabled";
pub const FEATURE_DISABLED: &str = "organization.feature_disabled";

pub struct FeatureEnabled {
    pub organization_id: OrganizationId,
    pub flag: FeatureFlag,
}

pub struct FeatureDisabled {
    pub organization_id: OrganizationId,
    pub flag: FeatureFlag,
}

impl DomainEvent for FeatureEnabled {
    fn name(&self) -> &'static str {
        FEATURE_ENABLED
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("organization", self.organization_id.0)
    }

    fn payload(&self) -> Value {
        json!({ "flag": self.flag.key() })
    }
}

impl DomainEvent for FeatureDisabled {
    fn name(&self) -> &'static str {
        FEATURE_DISABLED
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("organization", self.organization_id.0)
    }

    fn payload(&self) -> Value {
        json!({ "flag": self.flag.key() })
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::event_catalogue;

    fn organization() -> OrganizationId {
        OrganizationId(Uuid::from_u128(1))
    }

    #[test]
    fn the_events_carry_the_flag_key_and_name_the_organization() {
        let enabled = FeatureEnabled {
            organization_id: organization(),
            flag: FeatureFlag::DriveAi,
        };
        let disabled = FeatureDisabled {
            organization_id: organization(),
            flag: FeatureFlag::DriveAi,
        };

        assert_eq!(enabled.name(), "organization.feature_enabled");
        assert_eq!(disabled.name(), "organization.feature_disabled");
        assert_eq!(enabled.payload(), json!({ "flag": "drive_ai" }));
        assert_eq!(disabled.payload(), json!({ "flag": "drive_ai" }));
        assert_eq!(enabled.subject().id, Some(Uuid::from_u128(1)));
    }

    #[test]
    fn the_events_are_not_offered_as_workflow_triggers() {
        let catalogue = event_catalogue();

        assert!(catalogue.get(FEATURE_ENABLED, 1).is_none());
        assert!(catalogue.get(FEATURE_DISABLED, 1).is_none());
    }
}
