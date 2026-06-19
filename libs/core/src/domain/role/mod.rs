use std::ops::{BitAnd, BitOr, BitOrAssign};

use crate::domain::organization::OrganizationId;
use chrono::{DateTime, Utc};
pub use common::RoleId;
use serde::Serialize;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Permissions(pub i64);

impl Permissions {
    pub const NONE: Self = Permissions(0);

    pub const MANAGE_ORG: Self = Permissions(1 << 0);
    pub const MANAGE_MEMBERS: Self = Permissions(1 << 1);
    pub const MANAGE_ROLES: Self = Permissions(1 << 2);
    pub const MANAGE_CHANNELS: Self = Permissions(1 << 3);
    pub const MANAGE_WEBHOOKS: Self = Permissions(1 << 4);

    pub const ALL: Self = Permissions(i64::MAX);

    pub const fn contains(self, other: Permissions) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn bits(self) -> i64 {
        self.0
    }
}

impl BitOr for Permissions {
    type Output = Permissions;

    fn bitor(self, rhs: Self) -> Self::Output {
        Permissions(self.0 | rhs.0)
    }
}

impl BitAnd for Permissions {
    type Output = Permissions;

    fn bitand(self, rhs: Self) -> Self::Output {
        Permissions(self.0 & rhs.0)
    }
}

impl BitOrAssign for Permissions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone)]
pub struct Role {
    pub id: RoleId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub permissions: Permissions,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub const OWNER_ROLE_NAME: &str = "owner";
pub const ADMIN_ROLE_NAME: &str = "admin";
pub const MEMBER_ROLE_NAME: &str = "member";

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn role_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = RoleId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn role_id_rejects_invalid_uuid() {
        assert!(RoleId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn permissions_contains_subset() {
        let combined = Permissions::MANAGE_ORG | Permissions::MANAGE_MEMBERS;

        assert!(combined.contains(Permissions::MANAGE_ORG));
        assert!(combined.contains(Permissions::MANAGE_MEMBERS));
        assert!(!combined.contains(Permissions::MANAGE_ROLES));
    }

    #[test]
    fn permissions_all_contains_every_known_bit() {
        assert!(Permissions::ALL.contains(Permissions::MANAGE_ORG));
        assert!(Permissions::ALL.contains(Permissions::MANAGE_MEMBERS));
        assert!(Permissions::ALL.contains(Permissions::MANAGE_ROLES));
    }

    #[test]
    fn permissions_none_contains_only_none() {
        assert!(Permissions::NONE.is_empty());
        assert!(!Permissions::NONE.contains(Permissions::MANAGE_ORG));
        assert!(Permissions::NONE.contains(Permissions::NONE));
    }

    #[test]
    fn permissions_known_bits_have_stable_values() {
        // Append-only contract: never change these values.
        assert_eq!(Permissions::MANAGE_ORG.bits(), 1);
        assert_eq!(Permissions::MANAGE_MEMBERS.bits(), 2);
        assert_eq!(Permissions::MANAGE_ROLES.bits(), 4);
    }

    #[test]
    fn manage_channels_has_stable_bit_value() {
        assert_eq!(Permissions::MANAGE_CHANNELS.bits(), 8);
    }

    #[test]
    fn manage_webhooks_has_stable_bit_value() {
        assert_eq!(Permissions::MANAGE_WEBHOOKS.bits(), 16);
    }

    #[test]
    fn all_contains_manage_channels() {
        assert!(Permissions::ALL.contains(Permissions::MANAGE_CHANNELS));
    }

    #[test]
    fn all_contains_manage_webhooks() {
        assert!(Permissions::ALL.contains(Permissions::MANAGE_WEBHOOKS));
    }

    #[test]
    fn manage_channels_does_not_overlap_existing_bits() {
        assert!(!Permissions::MANAGE_ORG.contains(Permissions::MANAGE_CHANNELS));
        assert!(!Permissions::MANAGE_MEMBERS.contains(Permissions::MANAGE_CHANNELS));
        assert!(!Permissions::MANAGE_ROLES.contains(Permissions::MANAGE_CHANNELS));
    }

    #[test]
    fn manage_webhooks_does_not_overlap_existing_bits() {
        assert!(!Permissions::MANAGE_ORG.contains(Permissions::MANAGE_WEBHOOKS));
        assert!(!Permissions::MANAGE_MEMBERS.contains(Permissions::MANAGE_WEBHOOKS));
        assert!(!Permissions::MANAGE_ROLES.contains(Permissions::MANAGE_WEBHOOKS));
        assert!(!Permissions::MANAGE_CHANNELS.contains(Permissions::MANAGE_WEBHOOKS));
    }
}
