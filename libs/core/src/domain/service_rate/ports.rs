use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, ServiceRate, ServiceRateId};

#[cfg_attr(test, mockall::automock)]
pub trait ServiceRateRepository: Send {
    fn insert(
        &mut self,
        service_rate: &ServiceRate,
    ) -> impl Future<Output = Result<ServiceRate, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: ServiceRateId,
    ) -> impl Future<Output = Result<Option<ServiceRate>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<ServiceRate>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        service_rate: &ServiceRate,
    ) -> impl Future<Output = Result<ServiceRate, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: ServiceRateId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
