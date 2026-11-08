use common::CoreError;

use super::{Surcharge, SurchargeId};
use crate::OrganizationId;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SurchargeRepository: Send {
    fn lock_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Surcharge>, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: SurchargeId,
    ) -> impl Future<Output = Result<Option<Surcharge>, CoreError>> + Send;

    fn insert(
        &mut self,
        surcharge: &Surcharge,
    ) -> impl Future<Output = Result<Surcharge, CoreError>> + Send;

    fn update(
        &mut self,
        surcharge: &Surcharge,
    ) -> impl Future<Output = Result<Surcharge, CoreError>> + Send;
}
