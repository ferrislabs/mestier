use std::collections::HashSet;

use common::CoreError;

use crate::{
    Intervention, InterventionId, InterventionStatus, MaintenanceContract, MaintenanceContractId,
    MaintenanceLineId, MemberId, OrganizationId, TaskId,
};

/// What the field proposal needs to know about a task, without loading it
/// whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskMaintenanceContext {
    pub organization_id: OrganizationId,
    /// `None` when the task is not a maintenance task.
    pub line_id: Option<MaintenanceLineId>,
    pub assignee_member_ids: Vec<MemberId>,
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MaintenanceContractRepository: Send {
    /// Inserts the contract and its lines, in order.
    fn insert(
        &mut self,
        contract: &MaintenanceContract,
    ) -> impl Future<Output = Result<MaintenanceContract, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: MaintenanceContractId,
    ) -> impl Future<Output = Result<Option<MaintenanceContract>, CoreError>> + Send;

    fn find_by_line(
        &mut self,
        line_id: MaintenanceLineId,
    ) -> impl Future<Output = Result<Option<MaintenanceContract>, CoreError>> + Send;

    /// Ordered by reference, then creation.
    fn list_by_season(
        &mut self,
        organization_id: OrganizationId,
        year: i32,
        include_archived: bool,
    ) -> impl Future<Output = Result<Vec<MaintenanceContract>, CoreError>> + Send;

    /// Rewrites every field but `id`, `organization_id`, `project_id` and
    /// `created_at`, and replaces the lines: kept ids are updated, new ids
    /// inserted, missing ids deleted. Fails with `CoreError::Conflict` when
    /// a deleted line has interventions, so no history is dropped silently.
    fn update(
        &mut self,
        contract: &MaintenanceContract,
    ) -> impl Future<Output = Result<MaintenanceContract, CoreError>> + Send;

    /// The lines among `line_ids` carried by a task that is neither done
    /// nor cancelled.
    fn lines_with_open_task(
        &mut self,
        organization_id: OrganizationId,
        line_ids: &[MaintenanceLineId],
    ) -> impl Future<Output = Result<HashSet<MaintenanceLineId>, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait InterventionRepository: Send {
    fn insert(
        &mut self,
        intervention: &Intervention,
    ) -> impl Future<Output = Result<Intervention, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: InterventionId,
    ) -> impl Future<Output = Result<Option<Intervention>, CoreError>> + Send;

    /// Writes `status`, `resolved_by`, `resolved_at` and `note`.
    fn update_resolution(
        &mut self,
        intervention: &Intervention,
    ) -> impl Future<Output = Result<Intervention, CoreError>> + Send;

    /// Every intervention on these lines, any status, oldest first.
    fn list_for_lines(
        &mut self,
        line_ids: &[MaintenanceLineId],
    ) -> impl Future<Output = Result<Vec<Intervention>, CoreError>> + Send;

    /// A page of one organization's interventions in `status`, oldest
    /// first, with the total.
    fn list_by_status(
        &mut self,
        organization_id: OrganizationId,
        status: InterventionStatus,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Intervention>, u64), CoreError>> + Send;

    /// `None` when the task does not exist.
    fn find_task_context(
        &mut self,
        task_id: TaskId,
    ) -> impl Future<Output = Result<Option<TaskMaintenanceContext>, CoreError>> + Send;
}
