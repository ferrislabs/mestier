use common::CoreError;
use mestier_macros::transactional;

use crate::{
    AssignmentReport, AssignmentReportId, AssignmentReportResolution, MemberId, OrganizationId,
    application::MestierUseCase,
    domain::assignment_report::{
        commands::{
            AmendAssignmentReportCommand, ReportAssignmentCommand, ResolveAssignmentReportCommand,
            WithdrawAssignmentReportCommand,
        },
        service::AssignmentReportService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(assignment_report, emitter)]
    pub async fn report_assignment(
        &self,
        command: ReportAssignmentCommand,
    ) -> Result<AssignmentReport, CoreError> {
        let mut service = AssignmentReportService::new(assignment_report_repository, emitter);
        service.report_assignment(command).await
    }

    #[transactional(assignment_report, emitter)]
    pub async fn get_assignment_report(
        &self,
        id: AssignmentReportId,
    ) -> Result<AssignmentReport, CoreError> {
        let mut service = AssignmentReportService::new(assignment_report_repository, emitter);
        service.get_report(id).await
    }

    /// A page of `reported_by`'s own reports, most recent first — feeds
    /// `GET .../field/assignment-reports`.
    #[transactional(assignment_report, emitter)]
    pub async fn list_assignment_reports_by_reporter(
        &self,
        organization_id: OrganizationId,
        reported_by: MemberId,
        resolution: Option<AssignmentReportResolution>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<AssignmentReport>, u64), CoreError> {
        let mut service = AssignmentReportService::new(assignment_report_repository, emitter);
        service
            .list_reports_by_reporter(organization_id, reported_by, resolution, limit, offset)
            .await
    }

    /// A page of `organization_id`'s reports, most recent first — feeds the
    /// manager's list and the pending count on the planning views.
    #[transactional(assignment_report, emitter)]
    pub async fn list_assignment_reports_by_organization(
        &self,
        organization_id: OrganizationId,
        resolution: Option<AssignmentReportResolution>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<AssignmentReport>, u64), CoreError> {
        let mut service = AssignmentReportService::new(assignment_report_repository, emitter);
        service
            .list_reports_by_organization(organization_id, resolution, limit, offset)
            .await
    }

    #[transactional(assignment_report, emitter)]
    pub async fn amend_assignment_report(
        &self,
        command: AmendAssignmentReportCommand,
    ) -> Result<AssignmentReport, CoreError> {
        let mut service = AssignmentReportService::new(assignment_report_repository, emitter);
        service.amend_report(command).await
    }

    #[transactional(assignment_report, emitter)]
    pub async fn withdraw_assignment_report(
        &self,
        command: WithdrawAssignmentReportCommand,
    ) -> Result<(), CoreError> {
        let mut service = AssignmentReportService::new(assignment_report_repository, emitter);
        service.withdraw_report(command).await
    }

    #[transactional(assignment_report, emitter)]
    pub async fn resolve_assignment_report(
        &self,
        command: ResolveAssignmentReportCommand,
    ) -> Result<AssignmentReport, CoreError> {
        let mut service = AssignmentReportService::new(assignment_report_repository, emitter);
        service.resolve_report(command).await
    }
}
