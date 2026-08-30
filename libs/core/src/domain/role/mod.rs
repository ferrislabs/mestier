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

    // #391/#395: closes the last plain-membership reads. Customer and
    // invoice writes were already behind `MANAGE_CUSTOMERS` and (as of this
    // change) `MANAGE_INVOICES`; reading either list or record had no bit at
    // all, unlike every other business resource. `VIEW_CUSTOMERS` and
    // `VIEW_INVOICES` are separate bits, not implied by their `MANAGE_*`
    // counterpart, for the same reason `VIEW_COST`/`MANAGE_COST` are split:
    // an accountant may need to see every invoice without being trusted to
    // issue one.
    pub const VIEW_CUSTOMERS: Self = Permissions(1 << 15); // 32768
    pub const VIEW_INVOICES: Self = Permissions(1 << 16); // 65536
    // Invoices had no write permission at all before this — any member could
    // create, issue or cancel one under the plain membership gate. Deliberately
    // *not* granted to the seeded `member` role by default (see
    // `default_member_business_permissions`): unlike `MANAGE_CUSTOMERS` and
    // `MANAGE_QUOTES`, which member already lacked before this epic too, this
    // is a real behavior change for existing organizations, not a bit that
    // merely names an existing boundary — an explicit product call, not a
    // silent default.
    pub const MANAGE_INVOICES: Self = Permissions(1 << 17); // 131072

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

    /// Every named bit, paired with its own constant — the single source
    /// both the test suite's pairwise-disjointness check and #307's "what
    /// can this caller do" read off, so a ninth bit added later needs to
    /// be listed here exactly once to show up in both places.
    pub const NAMED: &[(&str, Permissions)] = &[
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
        ("VIEW_CUSTOMERS", Permissions::VIEW_CUSTOMERS),
        ("VIEW_INVOICES", Permissions::VIEW_INVOICES),
        ("MANAGE_INVOICES", Permissions::MANAGE_INVOICES),
    ];

    /// The names of every bit `self` carries — #307's "the caller's
    /// permissions, available anywhere" reads exactly this, over the HTTP
    /// boundary, to decide what to hide rather than gray out.
    pub fn granted_names(self) -> Vec<&'static str> {
        Permissions::NAMED
            .iter()
            .filter(|(_, bit)| self.contains(*bit))
            .map(|(name, _)| *name)
            .collect()
    }

    /// Inverse of [`Self::granted_names`]: reconstructs a `Permissions`
    /// value from bit names. Rejects any name absent from [`Self::NAMED`]
    /// rather than silently ignoring it — #308's role editor posts back
    /// exactly what it read, so an unknown name here means a stale client
    /// or a typo, not a bit to drop quietly.
    pub fn from_names<S: AsRef<str>>(names: &[S]) -> Result<Permissions, String> {
        let mut result = Permissions::NONE;
        for name in names {
            let name = name.as_ref();
            let (_, bit) = Permissions::NAMED
                .iter()
                .find(|(known, _)| *known == name)
                .ok_or_else(|| format!("unknown permission name: {name}"))?;
            result |= *bit;
        }
        Ok(result)
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
    /// Set once, at organization creation, for exactly the three roles
    /// [`OWNER_ROLE_NAME`]/[`ADMIN_ROLE_NAME`]/[`MEMBER_ROLE_NAME`] seed —
    /// never by a caller. #308: the earlier name-only match (see
    /// `migrations/20260829000002_grant_business_permissions.up.sql`'s own
    /// comment) let an org rename `owner` away and delete it right after,
    /// locking itself out. A seeded role's name is fixed and it cannot be
    /// deleted; its permissions stay editable — redefining what `admin`
    /// means is the point of #308.
    pub is_seeded: bool,
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
///
/// #395 adds `VIEW_CUSTOMERS`/`VIEW_INVOICES`/`MANAGE_INVOICES`: admin
/// already manages customers and quotes, so reading and now managing
/// invoices — the natural next step after a quote — follows the same
/// trust level, not a new one.
pub fn default_admin_business_permissions() -> Permissions {
    Permissions::VIEW_PLANNING
        | Permissions::MANAGE_PLANNING
        | Permissions::VIEW_REPORTS
        | Permissions::MANAGE_CUSTOMERS
        | Permissions::MANAGE_QUOTES
        | Permissions::MANAGE_REFERENCE
        | Permissions::VIEW_CUSTOMERS
        | Permissions::VIEW_INVOICES
        | Permissions::MANAGE_INVOICES
}

/// #304: the business bits a fresh organization's default `member` role
/// gets. Only the planning bits, matching what any member can already do
/// to the schedule today under the plain membership-only gate this epic
/// replaces — `VIEW_COST` would be exactly the payroll leak #283 closes,
/// and quotes/customers/reference stay with roles that manage the
/// business, not everyone in it. Same backfill-agreement requirement as
/// [`default_admin_business_permissions`].
///
/// #395 adds `VIEW_CUSTOMERS`/`VIEW_INVOICES` here — a member could already
/// read every customer and invoice under the plain membership gate this
/// epic replaces, so granting the read bits by default is not a new
/// capability, only naming one that already existed. `MANAGE_INVOICES` is
/// deliberately absent: unlike the two read bits, no member has ever needed
/// a permission to create or cancel an invoice before, because nothing
/// gated it — introducing the bit is the moment to also decide, explicitly,
/// that member does not hold it by default. An owner who wants members
/// invoicing directly grants it through a custom role.
pub fn default_member_business_permissions() -> Permissions {
    Permissions::VIEW_PLANNING
        | Permissions::MANAGE_PLANNING
        | Permissions::VIEW_CUSTOMERS
        | Permissions::VIEW_INVOICES
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
    /// function. `Permissions::NAMED` itself now also backs #307's "what can
    /// this caller do" read, so this test doubles as that list's own
    /// disjointness check.
    const NAMED_BITS: &[(&str, Permissions)] = Permissions::NAMED;

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

    #[test]
    fn granted_names_lists_only_the_bits_actually_held() {
        let combined = Permissions::VIEW_REPORTS | Permissions::VIEW_PLANNING;

        let names = combined.granted_names();

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"VIEW_REPORTS"));
        assert!(names.contains(&"VIEW_PLANNING"));
        assert!(!names.contains(&"VIEW_COST"));
    }

    #[test]
    fn granted_names_is_empty_for_none() {
        assert!(Permissions::NONE.granted_names().is_empty());
    }

    #[test]
    fn from_names_reconstructs_the_combined_bits() {
        let names = ["VIEW_REPORTS", "VIEW_PLANNING"];

        let permissions = Permissions::from_names(&names).unwrap();

        assert!(permissions.contains(Permissions::VIEW_REPORTS));
        assert!(permissions.contains(Permissions::VIEW_PLANNING));
        assert!(!permissions.contains(Permissions::VIEW_COST));
    }

    #[test]
    fn from_names_is_the_inverse_of_granted_names() {
        let original = Permissions::VIEW_REPORTS | Permissions::MANAGE_QUOTES;

        let roundtripped = Permissions::from_names(&original.granted_names()).unwrap();

        assert_eq!(roundtripped, original);
    }

    #[test]
    fn from_names_rejects_an_unknown_name() {
        let names = ["VIEW_REPORTS", "NOT_A_REAL_BIT"];

        assert!(Permissions::from_names(&names).is_err());
    }

    #[test]
    fn from_names_of_empty_list_is_none() {
        let names: [&str; 0] = [];

        assert_eq!(Permissions::from_names(&names).unwrap(), Permissions::NONE);
    }
}
