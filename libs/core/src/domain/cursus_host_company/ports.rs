use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{HostCompany, HostCompanyFilter, HostCompanyId, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CursusHostCompanyRepository: Send {
    fn insert(
        &mut self,
        company: &HostCompany,
    ) -> impl Future<Output = Result<HostCompany, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: HostCompanyId,
    ) -> impl Future<Output = Result<Option<HostCompany>, CoreError>> + Send;

    fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &HostCompanyFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<HostCompany>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        company: &HostCompany,
    ) -> impl Future<Output = Result<HostCompany, CoreError>> + Send;

    fn archive(
        &mut self,
        organization_id: OrganizationId,
        id: HostCompanyId,
        archived_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<HostCompany, CoreError>> + Send;
}
