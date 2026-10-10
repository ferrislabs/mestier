use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{DoeModel, DoeModelId, DoeSettings, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DoeModelRepository: Send {
    fn insert(
        &mut self,
        model: &DoeModel,
    ) -> impl Future<Output = Result<DoeModel, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: DoeModelId,
    ) -> impl Future<Output = Result<Option<DoeModel>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<DoeModel>, u64), CoreError>> + Send;

    /// Rewrites `name`, `description`, `sections`, `style` and `updated_at`.
    fn update(
        &mut self,
        model: &DoeModel,
    ) -> impl Future<Output = Result<DoeModel, CoreError>> + Send;

    fn set_archived_at(
        &mut self,
        id: DoeModelId,
        archived_at: Option<DateTime<Utc>>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DoeSettingsRepository: Send {
    fn find(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Option<DoeSettings>, CoreError>> + Send;

    fn upsert(
        &mut self,
        settings: &DoeSettings,
    ) -> impl Future<Output = Result<DoeSettings, CoreError>> + Send;
}
