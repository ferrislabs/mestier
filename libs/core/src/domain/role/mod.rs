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

    pub const VIEW_CHANNEL: Self = Permissions(1 << 5); // 32
    pub const SEND_MESSAGES: Self = Permissions(1 << 6); // 64

    // #283/#304: the business bits the bitfield never had. `VIEW_COST` and
    // `MANAGE_COST` are deliberately two bits, not one derived from
    // `MANAGE_MEMBERS`: an accountant reads every figure and sets none, and
    // a foreman who plans the week needs nobody's salary. `VIEW_REPORTS` is
    // its own bit rather than implied by `VIEW_COST`: the profitability and
    // worked-hours reports carry planned minutes too, which is a planning
    // concern independent of payroll.
    pub const VIEW_PLANNING: Self = Permissions(1 << 7); // 128
    pub const MANAGE_PLANNING: Self = Permissions(1 << 8); // 256
    pub const VIEW_COST: Self = Permissions(1 << 9); // 512
    pub const MANAGE_COST: Self = Permissions(1 << 10); // 1024
    pub const VIEW_REPORTS: Self = Permissions(1 << 11); // 2048
    pub const MANAGE_CUSTOMERS: Self = Permissions(1 << 12); // 4096
    pub const MANAGE_QUOTES: Self = Permissions(1 << 13); // 8192
    pub const MANAGE_REFERENCE: Self = Permissions(1 << 14); // 16384

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

/// #304: the business bits a fresh organization's default `admin` role
/// gets, in addition to its existing `MANAGE_MEMBERS`. The SQL backfill
/// (`migrations/20260829000002_grant_business_permissions.up.sql`) grants
/// the identical set to every existing `admin` row — two lists that drift
/// is how a role means one thing in production and another in a fresh
/// install, so a change here must be mirrored there.
///
/// `VIEW_COST`/`MANAGE_COST` are deliberately absent: admin already trusted
/// with running day-to-day operations does not imply trusted with payroll,
/// and bundling the two would recreate the leak #283 exists to close.
pub fn default_admin_business_permissions() -> Permissions {
    Permissions::VIEW_PLANNING
        | Permissions::MANAGE_PLANNING
        | Permissions::VIEW_REPORTS
        | Permissions::MANAGE_CUSTOMERS
        | Permissions::MANAGE_QUOTES
        | Permissions::MANAGE_REFERENCE
}

/// #304: the business bits a fresh organization's default `member` role
/// gets. Only the planning bits, matching what any member can already do
/// to the schedule today under the plain membership-only gate this epic
/// replaces — `VIEW_COST` would be exactly the payroll leak #283 closes,
/// and quotes/customers/reference stay with roles that manage the
/// business, not everyone in it. Same backfill-agreement requirement as
/// [`default_admin_business_permissions`].
pub fn default_member_business_permissions() -> Permissions {
    Permissions::VIEW_PLANNING | Permissions::MANAGE_PLANNING
}

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

    #[test]
    fn view_channel_has_stable_bit_value() {
        assert_eq!(Permissions::VIEW_CHANNEL.bits(), 32);
    }

    #[test]
    fn send_messages_has_stable_bit_value() {
        assert_eq!(Permissions::SEND_MESSAGES.bits(), 64);
    }

    #[test]
    fn view_channel_does_not_overlap_existing_bits() {
        assert!(!Permissions::MANAGE_ORG.contains(Permissions::VIEW_CHANNEL));
        assert!(!Permissions::MANAGE_MEMBERS.contains(Permissions::VIEW_CHANNEL));
        assert!(!Permissions::MANAGE_ROLES.contains(Permissions::VIEW_CHANNEL));
        assert!(!Permissions::MANAGE_CHANNELS.contains(Permissions::VIEW_CHANNEL));
        assert!(!Permissions::MANAGE_WEBHOOKS.contains(Permissions::VIEW_CHANNEL));
    }

    #[test]
    fn send_messages_does_not_overlap_existing_bits() {
        assert!(!Permissions::MANAGE_ORG.contains(Permissions::SEND_MESSAGES));
        assert!(!Permissions::MANAGE_MEMBERS.contains(Permissions::SEND_MESSAGES));
        assert!(!Permissions::MANAGE_ROLES.contains(Permissions::SEND_MESSAGES));
        assert!(!Permissions::MANAGE_CHANNELS.contains(Permissions::SEND_MESSAGES));
        assert!(!Permissions::MANAGE_WEBHOOKS.contains(Permissions::SEND_MESSAGES));
        assert!(!Permissions::VIEW_CHANNEL.contains(Permissions::SEND_MESSAGES));
    }

    #[test]
    fn all_contains_view_channel_and_send_messages() {
        assert!(Permissions::ALL.contains(Permissions::VIEW_CHANNEL));
        assert!(Permissions::ALL.contains(Permissions::SEND_MESSAGES));
    }

    /// The eight business bits #304 adds. One test each would just repeat the
    /// same "stable value" / "no overlap with any other named bit" shape the
    /// chat bits above already run five times over — past the point that
    /// duplication was buying anything, so this checks every named bit
    /// against every other one instead of writing an eighth near-identical
    /// function.
    const NAMED_BITS: &[(&str, Permissions)] = &[
        ("MANAGE_ORG", Permissions::MANAGE_ORG),
        ("MANAGE_MEMBERS", Permissions::MANAGE_MEMBERS),
        ("MANAGE_ROLES", Permissions::MANAGE_ROLES),
        ("MANAGE_CHANNELS", Permissions::MANAGE_CHANNELS),
        ("MANAGE_WEBHOOKS", Permissions::MANAGE_WEBHOOKS),
        ("VIEW_CHANNEL", Permissions::VIEW_CHANNEL),
        ("SEND_MESSAGES", Permissions::SEND_MESSAGES),
        ("VIEW_PLANNING", Permissions::VIEW_PLANNING),
        ("MANAGE_PLANNING", Permissions::MANAGE_PLANNING),
        ("VIEW_COST", Permissions::VIEW_COST),
        ("MANAGE_COST", Permissions::MANAGE_COST),
        ("VIEW_REPORTS", Permissions::VIEW_REPORTS),
        ("MANAGE_CUSTOMERS", Permissions::MANAGE_CUSTOMERS),
        ("MANAGE_QUOTES", Permissions::MANAGE_QUOTES),
        ("MANAGE_REFERENCE", Permissions::MANAGE_REFERENCE),
    ];

    #[test]
    fn business_permission_bits_have_stable_values() {
        // Append-only contract: never change these values.
        assert_eq!(Permissions::VIEW_PLANNING.bits(), 128);
        assert_eq!(Permissions::MANAGE_PLANNING.bits(), 256);
        assert_eq!(Permissions::VIEW_COST.bits(), 512);
        assert_eq!(Permissions::MANAGE_COST.bits(), 1024);
        assert_eq!(Permissions::VIEW_REPORTS.bits(), 2048);
        assert_eq!(Permissions::MANAGE_CUSTOMERS.bits(), 4096);
        assert_eq!(Permissions::MANAGE_QUOTES.bits(), 8192);
        assert_eq!(Permissions::MANAGE_REFERENCE.bits(), 16384);
    }

    #[test]
    fn all_named_bits_are_pairwise_disjoint() {
        for (i, (name_a, bit_a)) in NAMED_BITS.iter().enumerate() {
            for (name_b, bit_b) in &NAMED_BITS[i + 1..] {
                assert!(!bit_a.contains(*bit_b), "{name_a} and {name_b} overlap");
            }
        }
    }

    #[test]
    fn all_contains_every_named_bit() {
        for (name, bit) in NAMED_BITS {
            assert!(Permissions::ALL.contains(*bit), "ALL is missing {name}");
        }
    }
}
