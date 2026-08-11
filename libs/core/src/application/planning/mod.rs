use common::CoreError;
use mestier_macros::transactional;

use crate::{
    AvailabilityReport, DateRange, OrganizationId, PlanningEntry, PlanningView, TaskId, TimeRange,
    application::MestierUseCase,
    domain::{planning::service::PlanningService, task_label::ports::TaskLabelRepository},
};

mod tests;

impl MestierUseCase {
    /// `GET /planning`: resources, entries and work time over `range`.
    /// Read-only — never computes conflicts (see `get_availability`).
    ///
    /// Labels are attached here, after `PlanningService::get_planning`
    /// returns, rather than inside that domain service: `planning` and
    /// `task_label` are separate aggregates (see the planning module design
    /// doc), and this thin transactional seam is where `patch_task` already
    /// composes `TaskLabelRepository` for the same reason. One grouped
    /// query for every task entry in the window — never one per task.
    #[transactional(planning, employee, member, organization, task_label)]
    pub async fn get_planning(
        &self,
        organization_id: OrganizationId,
        range: DateRange,
    ) -> Result<PlanningView, CoreError> {
        let mut service = PlanningService::new(
            planning_repository,
            employee_repository,
            member_repository,
            organization_repository,
        );
        let mut view = service.get_planning(organization_id, range).await?;

        let task_ids: Vec<TaskId> = view
            .entries
            .iter()
            .filter_map(|entry| match entry {
                PlanningEntry::Task { id, .. } => Some(*id),
                PlanningEntry::Absence { .. } => None,
            })
            .collect();

        if !task_ids.is_empty() {
            let mut task_label_repository = task_label_repository;
            let mut labels_by_task = task_label_repository
                .list_labels_for_tasks(&task_ids)
                .await?;
            for entry in &mut view.entries {
                if let PlanningEntry::Task { id, labels, .. } = entry {
                    *labels = labels_by_task.remove(id).unwrap_or_default();
                }
            }
        }

        Ok(view)
    }

    /// `GET /planning/availability`: every resource of the organization,
    /// annotated with why it is or is not available for `window`. Never
    /// refuses anything — see invariant 1 in the planning module design
    /// doc.
    #[transactional(planning, employee, member, organization)]
    pub async fn get_availability(
        &self,
        organization_id: OrganizationId,
        window: TimeRange,
    ) -> Result<Vec<AvailabilityReport>, CoreError> {
        let mut service = PlanningService::new(
            planning_repository,
            employee_repository,
            member_repository,
            organization_repository,
        );
        service.get_availability(organization_id, window).await
    }
}
