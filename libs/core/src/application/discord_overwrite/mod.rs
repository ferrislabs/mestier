use common::CoreError;
use discord::domain::overwrite::ports::OverwriteRepository;
use discord::{
    Channel, ChannelId, ChannelPermissionOverwrite, ChannelRepository, DeleteChannelOverwrite,
    OrganizationId, OverwriteTarget, UpsertChannelOverwrite, UserId,
};
use mestier_macros::transactional;

use crate::application::{MestierUseCase, policy::resolve_org_permissions};
use crate::domain::member::ports::MemberRepository;
use crate::domain::role::{Permissions, ports::RoleRepository};

impl MestierUseCase {
    /// Resolves the caller's effective [`Permissions`] for a specific channel.
    ///
    /// 1. Load the channel → org_id.
    /// 2. Load the member + their role IDs; aggregate org-role bits.
    /// 3. base = aggregated | VIEW_CHANNEL | SEND_MESSAGES.
    /// 4. Load channel overwrites; pick EVERYONE / caller's roles / caller's MEMBER.
    /// 5. Call pure `discord::resolve_channel_permissions`; wrap as `Permissions`.
    #[transactional(member, role, overwrite, channel)]
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

        Ok(Permissions(effective))
    }

    /// Returns the channels in `organization_id` that `user_id` can view (VIEW_CHANNEL set).
    ///
    /// Loads all channels + the user's base bits + all overwrites for those channels;
    /// resolves per channel; keeps those where VIEW_CHANNEL is set.
    #[transactional(member, role, overwrite, channel)]
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

        let mut visible = Vec::new();
        for ch in all_channels {
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
