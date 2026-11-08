use common::CoreError;
use discord::domain::overwrite::ports::OverwriteRepository;
use discord::{
    CategoryAccessRepository, Channel, ChannelId, ChannelPermissionOverwrite, ChannelRepository,
    ChannelService, ChannelType, CreateChannelCommand, DeleteChannelOverwrite,
    DirectChannelRepository, OrganizationId, OverwriteTarget, RoleId, UpsertChannelOverwrite,
    UserId,
};
use mestier_macros::transactional;

use crate::application::{MestierUseCase, policy::resolve_org_permissions};
use crate::domain::member::ports::MemberRepository;
use crate::domain::role::{Permissions, ports::RoleRepository};

#[cfg(test)]
mod tests;

fn without_access(effective: Permissions) -> Permissions {
    Permissions(
        effective.bits() & !(Permissions::VIEW_CHANNEL.bits() | Permissions::SEND_MESSAGES.bits()),
    )
}

/// What the kind of channel takes away, whatever the overwrites say: nobody
/// types in a voice channel, and only those who manage channels write in an
/// announcement channel.
fn restrict_by_channel_type(channel_type: ChannelType, effective: Permissions) -> Permissions {
    let without_sending = Permissions(effective.bits() & !Permissions::SEND_MESSAGES.bits());
    match channel_type {
        ChannelType::Voice => without_sending,
        ChannelType::Announcement if !effective.contains(Permissions::MANAGE_CHANNELS) => {
            without_sending
        }
        _ => effective,
    }
}

impl MestierUseCase {
    /// Resolves the caller's effective [`Permissions`] for a specific channel.
    ///
    /// 1. Load the channel → org_id.
    /// 2. Load the member + their role IDs; aggregate org-role bits.
    /// 3. base = aggregated | VIEW_CHANNEL | SEND_MESSAGES.
    /// 4. Load channel overwrites; pick EVERYONE / caller's roles / caller's MEMBER.
    /// 5. Call pure `discord::resolve_channel_permissions`; wrap as `Permissions`.
    #[transactional(member, role, overwrite, channel, direct_channel, category_access)]
    pub async fn resolve_channel_permissions(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> Result<Permissions, CoreError> {
        let mut channel_repository = channel_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let channel = channel_repository
            .find_by_id(channel_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let org_id = channel.organization_id;

        let member = member_repository
            .find_by_org_and_user(org_id, user_id)
            .await?
            .ok_or(CoreError::Forbidden {
                reason: Some("not a member of this organization".to_owned()),
            })?;

        // A direct channel belongs to its two people and to nobody else, whatever
        // their org roles or overwrites say. Anyone else gets `NotFound`, so a
        // conversation cannot be told apart from one that does not exist.
        if channel.channel_type == ChannelType::Direct {
            let mut direct_channel_repository = direct_channel_repository;
            let direct = direct_channel_repository
                .find_by_channel(channel_id)
                .await?
                .ok_or(CoreError::NotFound)?;
            return if direct.pair.includes(user_id) {
                Ok(Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES)
            } else {
                Err(CoreError::NotFound)
            };
        }

        let member_role_ids = member_repository.list_role_ids(member.id).await?;
        let org_roles = role_repository.list_by_organization(org_id).await?;
        let aggregated = resolve_org_permissions(&member, &member_role_ids, &org_roles);

        let base = (aggregated | Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES).bits();

        let all_overwrites = overwrite_repository.list_for_channel(channel_id).await?;

        let everyone = all_overwrites
            .iter()
            .find(|o| matches!(o.target, OverwriteTarget::Everyone));

        let role_overwrites: Vec<ChannelPermissionOverwrite> = all_overwrites
            .iter()
            .filter(
                |o| matches!(&o.target, OverwriteTarget::Role(r) if member_role_ids.contains(r)),
            )
            .cloned()
            .collect();

        let member_overwrite = all_overwrites
            .iter()
            .find(|o| matches!(&o.target, OverwriteTarget::Member(u) if *u == user_id));

        let effective = discord::resolve_channel_permissions(
            base,
            everyone,
            &role_overwrites,
            member_overwrite,
        );

        // A private category is a gate on every channel in it, whatever the
        // channel's own overwrites grant.
        let mut effective = Permissions(effective);
        if let Some(category_id) = channel.category_id {
            let mut category_access_repository = category_access_repository;
            if let Some(access) = category_access_repository.find(category_id).await?
                && !access.admits(user_id, &member_role_ids)
            {
                effective = without_access(effective);
            }
        }

        Ok(restrict_by_channel_type(channel.channel_type, effective))
    }

    /// Returns the channels in `organization_id` that `user_id` can view (VIEW_CHANNEL set).
    ///
    /// Loads all channels + the user's base bits + all overwrites for those channels;
    /// resolves per channel; keeps those where VIEW_CHANNEL is set.
    #[transactional(member, role, overwrite, channel, category_access)]
    pub async fn list_visible_channels(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<Channel>, CoreError> {
        let mut channel_repository = channel_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let member = member_repository
            .find_by_org_and_user(organization_id, user_id)
            .await?
            .ok_or(CoreError::Forbidden {
                reason: Some("not a member of this organization".to_owned()),
            })?;

        let member_role_ids = member_repository.list_role_ids(member.id).await?;
        let org_roles = role_repository
            .list_by_organization(organization_id)
            .await?;
        let aggregated = resolve_org_permissions(&member, &member_role_ids, &org_roles);
        let base = (aggregated | Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES).bits();

        let all_channels = channel_repository
            .list_by_organization(organization_id)
            .await?;

        let mut category_access_repository = category_access_repository;
        let private_categories = category_access_repository
            .list_for_organization(organization_id)
            .await?;

        let mut visible = Vec::new();
        for ch in all_channels {
            if let Some(access) = ch
                .category_id
                .and_then(|category_id| private_categories.get(&category_id))
                && !access.admits(user_id, &member_role_ids)
            {
                continue;
            }

            let overwrites = overwrite_repository.list_for_channel(ch.id).await?;

            let everyone = overwrites
                .iter()
                .find(|o| matches!(o.target, OverwriteTarget::Everyone));

            let role_overwrites: Vec<ChannelPermissionOverwrite> = overwrites
                .iter()
                .filter(|o| {
                    matches!(&o.target, OverwriteTarget::Role(r) if member_role_ids.contains(r))
                })
                .cloned()
                .collect();

            let member_overwrite = overwrites
                .iter()
                .find(|o| matches!(&o.target, OverwriteTarget::Member(u) if *u == user_id));

            let effective = discord::resolve_channel_permissions(
                base,
                everyone,
                &role_overwrites,
                member_overwrite,
            );

            if Permissions(effective).contains(Permissions::VIEW_CHANNEL) {
                visible.push(ch);
            }
        }

        Ok(visible)
    }

    /// Creates a channel only `role_ids` (and its creator) can see and write in.
    ///
    /// The channel and its overwrites land in one transaction, so a private
    /// channel is never visible, even for an instant, before it is closed.
    /// `creator` gets a member overwrite of their own: whatever roles they hold,
    /// they cannot lock themselves out of what they just made.
    #[transactional(channel, overwrite, role, events)]
    pub async fn create_private_channel(
        &self,
        command: CreateChannelCommand,
        creator: UserId,
        role_ids: Vec<RoleId>,
    ) -> Result<Channel, CoreError> {
        if role_ids.is_empty() {
            return Err(CoreError::Conflict(
                "a private channel needs at least one role".to_owned(),
            ));
        }
        let organization_id = command.organization_id;
        let mut role_repository = role_repository;
        let org_roles = role_repository
            .list_by_organization(organization_id)
            .await?;
        if let Some(unknown) = role_ids
            .iter()
            .find(|role_id| !org_roles.iter().any(|role| role.id == **role_id))
        {
            return Err(CoreError::Conflict(format!(
                "role {} does not belong to this organization",
                unknown.0
            )));
        }

        let mut service = ChannelService::new(channel_repository, &events);
        let channel = service.create_channel(command).await?;

        let access = (Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES).bits();
        let overwrite = |target, allow, deny| UpsertChannelOverwrite {
            channel_id: channel.id,
            organization_id,
            target,
            allow,
            deny,
        };
        overwrite_repository
            .upsert(overwrite(OverwriteTarget::Everyone, 0, access))
            .await?;
        for role_id in role_ids {
            overwrite_repository
                .upsert(overwrite(OverwriteTarget::Role(role_id), access, 0))
                .await?;
        }
        overwrite_repository
            .upsert(overwrite(OverwriteTarget::Member(creator), access, 0))
            .await?;

        Ok(channel)
    }

    /// Upserts a channel permission overwrite (EVERYONE, ROLE, or MEMBER).
    #[transactional(overwrite)]
    pub async fn upsert_channel_overwrite(
        &self,
        command: UpsertChannelOverwrite,
    ) -> Result<ChannelPermissionOverwrite, CoreError> {
        overwrite_repository.upsert(command).await
    }

    /// Hard-deletes a channel permission overwrite.
    #[transactional(overwrite)]
    pub async fn delete_channel_overwrite(
        &self,
        command: DeleteChannelOverwrite,
    ) -> Result<(), CoreError> {
        overwrite_repository
            .delete(command.channel_id, command.target)
            .await
    }

    /// Lists all channel permission overwrites for a channel.
    #[transactional(overwrite)]
    pub async fn list_channel_overwrites(
        &self,
        channel_id: ChannelId,
    ) -> Result<Vec<ChannelPermissionOverwrite>, CoreError> {
        overwrite_repository.list_for_channel(channel_id).await
    }
}

#[cfg(test)]
mod restriction_tests {
    use super::*;

    const MEMBER: Permissions =
        Permissions(Permissions::VIEW_CHANNEL.0 | Permissions::SEND_MESSAGES.0);
    const MANAGER: Permissions = Permissions(MEMBER.0 | Permissions::MANAGE_CHANNELS.0);

    #[test]
    fn a_text_channel_is_left_as_resolved() {
        assert_eq!(restrict_by_channel_type(ChannelType::Text, MEMBER), MEMBER);
    }

    #[test]
    fn nobody_sends_in_a_voice_channel() {
        let restricted = restrict_by_channel_type(ChannelType::Voice, MANAGER);

        assert!(restricted.contains(Permissions::VIEW_CHANNEL));
        assert!(!restricted.contains(Permissions::SEND_MESSAGES));
    }

    #[test]
    fn only_a_manager_sends_in_an_announcement_channel() {
        let member = restrict_by_channel_type(ChannelType::Announcement, MEMBER);
        let manager = restrict_by_channel_type(ChannelType::Announcement, MANAGER);

        assert!(member.contains(Permissions::VIEW_CHANNEL));
        assert!(!member.contains(Permissions::SEND_MESSAGES));
        assert!(manager.contains(Permissions::SEND_MESSAGES));
    }
}
