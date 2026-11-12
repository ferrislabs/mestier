use common::CoreError;

use crate::{FeatureActivation, FeatureFlag, OrganizationId, UserId};

pub trait FeatureFlagRepository: Send {
    fn list_enabled(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<FeatureActivation>, CoreError>> + Send;

    fn is_enabled(
        &mut self,
        organization_id: OrganizationId,
        flag: FeatureFlag,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn enable(
        &mut self,
        organization_id: OrganizationId,
        flag: FeatureFlag,
        by: UserId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn disable(
        &mut self,
        organization_id: OrganizationId,
        flag: FeatureFlag,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
