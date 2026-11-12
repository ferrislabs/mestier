use common::CoreError;
use mestier_macros::transactional;

use crate::{
    FeatureFlag, OrganizationId, application::MestierUseCase,
    domain::feature_flag::ports::FeatureFlagRepository,
};

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
}
