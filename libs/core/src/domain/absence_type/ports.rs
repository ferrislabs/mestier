use common::CoreError;

use super::{AbsenceType, AbsenceTypeId};
use crate::OrganizationId;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait AbsenceTypeRepository: Send {
    fn lock_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<AbsenceType>, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: AbsenceTypeId,
    ) -> impl Future<Output = Result<Option<AbsenceType>, CoreError>> + Send;

    fn insert(
        &mut self,
        absence_type: &AbsenceType,
    ) -> impl Future<Output = Result<AbsenceType, CoreError>> + Send;

    fn update(
        &mut self,
        absence_type: &AbsenceType,
    ) -> impl Future<Output = Result<AbsenceType, CoreError>> + Send;
}
