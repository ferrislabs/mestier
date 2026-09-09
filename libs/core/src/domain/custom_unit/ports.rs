use common::CoreError;

use crate::{CustomUnit, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CustomUnitRepository: Send {
    fn insert(
        &mut self,
        custom_unit: &CustomUnit,
    ) -> impl Future<Output = Result<CustomUnit, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<CustomUnit>, CoreError>> + Send;
}
