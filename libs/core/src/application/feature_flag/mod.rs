use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    FeatureDisabled, FeatureEnabled, FeatureFlag, OrganizationId, SetSelfServiceFeatureCommand,
    application::{MestierUseCase, policy},
    domain::{
        feature_flag::ports::FeatureFlagRepository, member::ports::MemberRepository,
        organization::ports::OrganizationRepository,
    },
};

#[cfg(test)]
mod tests;

impl MestierUseCase {
    #[transactional(feature_flag)]
    pub async fn enabled_features(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<FeatureFlag>, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;

        let activations = feature_flag_repository
            .list_enabled(organization_id)
            .await?;
        Ok(activations.into_iter().map(|a| a.flag).collect())
    }

    #[transactional(feature_flag)]
    pub async fn is_feature_enabled(
        &self,
        organization_id: OrganizationId,
        flag: FeatureFlag,
    ) -> Result<bool, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;

        feature_flag_repository
            .is_enabled(organization_id, flag)
            .await
    }

    pub async fn set_self_service_feature(
        &self,
        command: SetSelfServiceFeatureCommand,
    ) -> Result<bool, CoreError> {
        self.acting_as(command.user_id)
            .apply_self_service_feature(command)
            .await
    }

    #[transactional(organization, member, role, feature_flag, authz, emitter)]
    async fn apply_self_service_feature(
        &self,
        command: SetSelfServiceFeatureCommand,
    ) -> Result<bool, CoreError> {
        let mut organization_repository = organization_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let SetSelfServiceFeatureCommand {
            actor,
            user_id,
            organization_id,
            flag,
            enabled,
        } = command;

        let organization = organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        member_repository
            .find_by_org_and_user(organization_id, user_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let flag = flag.require_self_service()?;
        let actor = policy::enrich_for_organization(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            authz,
            &actor,
            "organization.update",
            Resource::new("organization", organization_id.0.to_string())
                .with_property("mestier.slug", organization.slug),
        )
        .await?;

        let changed = if enabled {
            feature_flag_repository
                .enable(organization_id, flag, user_id)
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
}
