use crate::domain::ChannelPermissionOverwrite;

pub fn resolve_channel_permissions(
    base: i64,
    everyone: Option<&ChannelPermissionOverwrite>,
    role_overwrites: &[ChannelPermissionOverwrite],
    member: Option<&ChannelPermissionOverwrite>,
) -> i64 {
    let mut perms = base;
    if let Some(e) = everyone {
        perms = (perms & !e.deny) | e.allow;
    }
    let role_allow = role_overwrites.iter().fold(0_i64, |a, o| a | o.allow);
    let role_deny = role_overwrites.iter().fold(0_i64, |a, o| a | o.deny);
    perms = (perms & !role_deny) | role_allow;
    if let Some(m) = member {
        perms = (perms & !m.deny) | m.allow;
    }
    perms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ChannelPermissionOverwrite, OverwriteTarget};
    use crate::{ChannelId, OverwriteId};
    use chrono::Utc;
    use common::OrganizationId;
    use uuid::Uuid;

    // Bit constants mirroring the design spec §3 (used without importing `core::Permissions`).
    const VIEW_CHANNEL: i64 = 1 << 5; // 32
    const SEND_MESSAGES: i64 = 1 << 6; // 64
    const DEFAULT_BASE: i64 = VIEW_CHANNEL | SEND_MESSAGES; // 96

    fn make_overwrite(
        channel_id: ChannelId,
        target: OverwriteTarget,
        allow: i64,
        deny: i64,
    ) -> ChannelPermissionOverwrite {
        ChannelPermissionOverwrite {
            id: OverwriteId(Uuid::new_v4()),
            channel_id,
            organization_id: OrganizationId(Uuid::new_v4()),
            target,
            allow,
            deny,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Test 1: No overwrites — base bits pass through unchanged.
    #[test]
    fn base_default_passthrough_no_overwrites() {
        let result = resolve_channel_permissions(DEFAULT_BASE, None, &[], None);
        assert_eq!(result, DEFAULT_BASE, "no overwrites must preserve the base");
    }

    /// Test 2: EVERYONE deny removes a bit from the base.
    #[test]
    fn everyone_deny_removes_view_channel_bit() {
        let channel_id = ChannelId(Uuid::new_v4());
        let everyone = make_overwrite(channel_id, OverwriteTarget::Everyone, 0, VIEW_CHANNEL);

        let result = resolve_channel_permissions(DEFAULT_BASE, Some(&everyone), &[], None);
        assert_eq!(result & VIEW_CHANNEL, 0, "VIEW_CHANNEL must be denied");
        assert_eq!(
            result & SEND_MESSAGES,
            SEND_MESSAGES,
            "SEND_MESSAGES must be unaffected"
        );
    }

    /// Test 3: A ROLE allow re-grants VIEW_CHANNEL over an EVERYONE deny.
    #[test]
    fn role_allow_regrants_over_everyone_deny() {
        let channel_id = ChannelId(Uuid::new_v4());
        // EVERYONE denies VIEW_CHANNEL
        let everyone = make_overwrite(channel_id, OverwriteTarget::Everyone, 0, VIEW_CHANNEL);
        // A role the caller holds re-grants VIEW_CHANNEL
        use common::RoleId;
        let role_overwrite = make_overwrite(
            channel_id,
            OverwriteTarget::Role(RoleId(Uuid::new_v4())),
            VIEW_CHANNEL,
            0,
        );

        let result =
            resolve_channel_permissions(DEFAULT_BASE, Some(&everyone), &[role_overwrite], None);
        assert_eq!(
            result & VIEW_CHANNEL,
            VIEW_CHANNEL,
            "role allow must re-grant VIEW_CHANNEL"
        );
    }

    /// Test 4: Combined role overwrites — allows are OR-ed, denies are OR-ed.
    #[test]
    fn combined_role_overwrites_or_allows_and_denies() {
        let channel_id = ChannelId(Uuid::new_v4());
        use common::RoleId;

        // Role A allows VIEW_CHANNEL, role B denies SEND_MESSAGES
        let role_a = make_overwrite(
            channel_id,
            OverwriteTarget::Role(RoleId(Uuid::new_v4())),
            VIEW_CHANNEL,
            0,
        );
        let role_b = make_overwrite(
            channel_id,
            OverwriteTarget::Role(RoleId(Uuid::new_v4())),
            0,
            SEND_MESSAGES,
        );

        // Base already has VIEW_CHANNEL | SEND_MESSAGES
        let result = resolve_channel_permissions(DEFAULT_BASE, None, &[role_a, role_b], None);
        assert_eq!(
            result & VIEW_CHANNEL,
            VIEW_CHANNEL,
            "VIEW_CHANNEL allowed by role_a"
        );
        assert_eq!(result & SEND_MESSAGES, 0, "SEND_MESSAGES denied by role_b");
    }

    /// Test 5: MEMBER allow beats a role deny.
    #[test]
    fn member_allow_beats_role_deny() {
        let channel_id = ChannelId(Uuid::new_v4());
        use common::{RoleId, UserId};

        // Role denies SEND_MESSAGES
        let role_overwrite = make_overwrite(
            channel_id,
            OverwriteTarget::Role(RoleId(Uuid::new_v4())),
            0,
            SEND_MESSAGES,
        );
        // Member-specific overwrite re-grants it
        let member_overwrite = make_overwrite(
            channel_id,
            OverwriteTarget::Member(UserId(Uuid::new_v4())),
            SEND_MESSAGES,
            0,
        );

        let result = resolve_channel_permissions(
            DEFAULT_BASE,
            None,
            &[role_overwrite],
            Some(&member_overwrite),
        );
        assert_eq!(
            result & SEND_MESSAGES,
            SEND_MESSAGES,
            "member allow must beat role deny"
        );
    }

    /// Test 6: MEMBER deny beats a role allow.
    #[test]
    fn member_deny_beats_role_allow() {
        let channel_id = ChannelId(Uuid::new_v4());
        use common::{RoleId, UserId};

        // Role allows VIEW_CHANNEL explicitly (base already has it, but explicit is cleaner)
        let role_overwrite = make_overwrite(
            channel_id,
            OverwriteTarget::Role(RoleId(Uuid::new_v4())),
            VIEW_CHANNEL,
            0,
        );
        // Member-specific overwrite denies VIEW_CHANNEL
        let member_overwrite = make_overwrite(
            channel_id,
            OverwriteTarget::Member(UserId(Uuid::new_v4())),
            0,
            VIEW_CHANNEL,
        );

        let result = resolve_channel_permissions(
            DEFAULT_BASE,
            None,
            &[role_overwrite],
            Some(&member_overwrite),
        );
        assert_eq!(result & VIEW_CHANNEL, 0, "member deny must beat role allow");
    }
}
