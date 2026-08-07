use common::CoreError;
use mestier_macros::transactional;

use crate::{
    AvailabilityReport, DateRange, OrganizationId, PlanningView, TimeRange,
    application::MestierUseCase, domain::planning::service::PlanningService,
};

mod tests;

impl MestierUseCase {
    /// `GET /planning`: resources, entries and work time over `range`.
    /// Read-only — never computes conflicts (see `get_availability`).
    #[transactional(planning, employee, member, organization, user)]
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
            user_repository,
        );
        service.get_planning(organization_id, range).await
    }

    /// `GET /planning/availability`: every resource of the organization,
    /// annotated with why it is or is not available for `window`. Never
    /// refuses anything — see invariant 1 in the planning module design
    /// doc.
    #[transactional(planning, employee, member, organization, user)]
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
            user_repository,
        );
        service.get_availability(organization_id, window).await
    }
}
