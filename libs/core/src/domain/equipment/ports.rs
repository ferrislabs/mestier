use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{Equipment, EquipmentId, OrganizationId};

#[cfg_attr(test, mockall::automock)]
pub trait EquipmentRepository: Send {
    fn insert(
        &mut self,
        equipment: &Equipment,
    ) -> impl Future<Output = Result<Equipment, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: EquipmentId,
    ) -> impl Future<Output = Result<Option<Equipment>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Equipment>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        equipment: &Equipment,
    ) -> impl Future<Output = Result<Equipment, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: EquipmentId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
