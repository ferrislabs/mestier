use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    FeatureChange, OrganizationId, OrganizationOverview, OrganizationSummary, PlatformStats,
    UserSummary,
};

pub trait PlatformAdminRepository: Send {
    fn list_organizations(
        &mut self,
        search: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<OrganizationSummary>, u64), CoreError>> + Send;

    fn organization_overview(
        &mut self,
        id: OrganizationId,
    ) -> impl Future<Output = Result<Option<OrganizationOverview>, CoreError>> + Send;

    fn feature_history(
        &mut self,
        id: OrganizationId,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<FeatureChange>, CoreError>> + Send;

    fn list_users(
        &mut self,
        search: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<UserSummary>, u64), CoreError>> + Send;

    fn stats(
        &mut self,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<PlatformStats, CoreError>> + Send;
}
