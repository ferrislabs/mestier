use chrono::Utc;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    FeatureDisabled, FeatureEnabled, FeatureFlag, HISTORY_LIMIT, OrganizationDetail,
    OrganizationFeature, OrganizationId, OrganizationSummary, PlatformAdmin, PlatformStats, UserId,
    UserSummary,
    application::MestierUseCase,
    domain::{
        feature_flag::ports::FeatureFlagRepository, organization::ports::OrganizationRepository,
        platform::ports::PlatformAdminRepository, user::ports::UserRepository,
    },
};

impl MestierUseCase {
    #[transactional(platform_admin)]
    pub async fn list_organizations_for_admin(
        &self,
        _: PlatformAdmin,
        search: Option<String>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<OrganizationSummary>, u64), CoreError> {
        let mut platform_admin_repository = platform_admin_repository;

        platform_admin_repository
            .list_organizations(search.as_deref(), limit, offset)
            .await
    }

    #[transactional(platform_admin, feature_flag, user)]
    pub async fn get_organization_for_admin(
        &self,
        _: PlatformAdmin,
        organization_id: OrganizationId,
    ) -> Result<OrganizationDetail, CoreError> {
        let mut platform_admin_repository = platform_admin_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let mut user_repository = user_repository;

        let overview = platform_admin_repository
            .organization_overview(organization_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let activations = feature_flag_repository
            .list_enabled(organization_id)
            .await?;
        let history = platform_admin_repository
            .feature_history(organization_id, HISTORY_LIMIT)
            .await?;

        let mut features = Vec::with_capacity(FeatureFlag::ALL.len());
        for flag in FeatureFlag::ALL {
            let activation = activations.iter().find(|a| a.flag == *flag).copied();
            let enabled_by_name = match activation.and_then(|a| a.enabled_by) {
                Some(user_id) => user_repository
                    .find_by_id(user_id)
                    .await?
                    .map(|user| user.name),
                None => None,
            };
            features.push(OrganizationFeature {
                flag: *flag,
                activation,
                enabled_by_name,
            });
        }

        Ok(OrganizationDetail {
            overview,
            features,
            history,
        })
    }

    pub async fn set_organization_feature(
        &self,
        admin: PlatformAdmin,
        actor: UserId,
        organization_id: OrganizationId,
        flag: FeatureFlag,
        enabled: bool,
    ) -> Result<bool, CoreError> {
        self.acting_as(actor)
            .apply_organization_feature(admin, actor, organization_id, flag, enabled)
            .await
    }

    #[transactional(organization, feature_flag, emitter)]
    async fn apply_organization_feature(
        &self,
        _: PlatformAdmin,
        actor: UserId,
        organization_id: OrganizationId,
        flag: FeatureFlag,
        enabled: bool,
    ) -> Result<bool, CoreError> {
        let mut organization_repository = organization_repository;
        let mut feature_flag_repository = feature_flag_repository;

        organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let changed = if enabled {
            feature_flag_repository
                .enable(organization_id, flag, actor)
                .await?
        } else {
            feature_flag_repository
                .disable(organization_id, flag)
                .await?
        };

        if changed {
            if enabled {
                emitter.emit(
                    organization_id,
                    &FeatureEnabled {
                        organization_id,
                        flag,
                    },
                )?;
            } else {
                emitter.emit(
                    organization_id,
                    &FeatureDisabled {
                        organization_id,
                        flag,
                    },
                )?;
            }
        }

        Ok(changed)
    }

    #[transactional(platform_admin)]
    pub async fn list_users_for_admin(
        &self,
        _: PlatformAdmin,
        search: Option<String>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<UserSummary>, u64), CoreError> {
        let mut platform_admin_repository = platform_admin_repository;

        platform_admin_repository
            .list_users(search.as_deref(), limit, offset)
            .await
    }

    #[transactional(platform_admin)]
    pub async fn platform_stats(&self, _: PlatformAdmin) -> Result<PlatformStats, CoreError> {
        let mut platform_admin_repository = platform_admin_repository;

        platform_admin_repository.stats(Utc::now()).await
    }

    pub fn list_feature_catalogue(&self, _: PlatformAdmin) -> &'static [FeatureFlag] {
        FeatureFlag::ALL
    }
}

#[cfg(test)]
mod tests;
