use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Employee, OrganizationId, WorkOrder, WorkOrderId,
    application::MestierUseCase,
    domain::work_order::{
        commands::{CreateWorkOrderCommand, PatchWorkOrderCommand},
        service::WorkOrderService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(work_order, employee, user, member)]
    pub async fn create_work_order(
        &self,
        command: CreateWorkOrderCommand,
    ) -> Result<WorkOrder, CoreError> {
        let mut service = WorkOrderService::new(
            work_order_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service.create_work_order(command).await
    }

    #[transactional(work_order, employee, user, member)]
    pub async fn get_work_order(&self, id: WorkOrderId) -> Result<WorkOrder, CoreError> {
        let mut service = WorkOrderService::new(
            work_order_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service.get_work_order(id).await
    }

    #[transactional(work_order, employee, user, member)]
    pub async fn list_work_orders(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<WorkOrder>, u64), CoreError> {
        let mut service = WorkOrderService::new(
            work_order_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service
            .list_work_orders(organization_id, limit, offset)
            .await
    }

    /// Reschedules and reassigns a work order in one transaction: either
    /// every write here (the schedule/status/title/note edits, the full
    /// assignment replacement, and any on-the-fly employee record) lands
    /// together, or the whole `PATCH` rolls back.
    #[transactional(work_order, employee, user, member)]
    pub async fn patch_work_order(
        &self,
        command: PatchWorkOrderCommand,
    ) -> Result<(WorkOrder, Vec<Employee>), CoreError> {
        let mut service = WorkOrderService::new(
            work_order_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service.patch_work_order(command).await
    }

    #[transactional(work_order, employee, user, member)]
    pub async fn soft_delete_work_order(&self, id: WorkOrderId) -> Result<(), CoreError> {
        let mut service = WorkOrderService::new(
            work_order_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service.soft_delete_work_order(id).await
    }
}
