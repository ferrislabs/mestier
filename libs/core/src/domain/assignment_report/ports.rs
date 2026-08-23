use common::CoreError;

use crate::{
    AssignmentReport, AssignmentReportId, MemberId, OrganizationId, TaskAssignmentId, TaskId,
    domain::assignment_report::AssignmentReportResolution,
};

/// What the service needs to know about the assignment a report targets,
/// without loading the whole task and its full assignment list — mirrors
/// `TaskCommentRepository`'s own preference for a narrow, purpose-built read
/// over reusing `TaskRepository::find_by_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssignmentContext {
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub member_id: MemberId,
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait AssignmentReportRepository: Send {
    /// Resolves a `task_assignment_id` to the organization and member it
    /// belongs to, so the service can check "only the assignee may report on
    /// their own assignment" before ever writing a row. `None` when the
    /// assignment does not exist.
    fn find_assignment_context(
        &mut self,
        task_assignment_id: TaskAssignmentId,
    ) -> impl Future<Output = Result<Option<AssignmentContext>, CoreError>> + Send;

    fn insert(
        &mut self,
        report: &AssignmentReport,
    ) -> impl Future<Output = Result<AssignmentReport, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: AssignmentReportId,
    ) -> impl Future<Output = Result<Option<AssignmentReport>, CoreError>> + Send;

    /// A page of one reporter's own reports, most recent first, together
    /// with the total — mirrors `TaskCommentRepository::list_by_task`. Feeds
    /// `GET .../field/assignment-reports`, which shows a worker their own
    /// history, resolved included.
    fn list_by_reporter(
        &mut self,
        organization_id: OrganizationId,
        reported_by: MemberId,
        resolution: Option<AssignmentReportResolution>,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<AssignmentReport>, u64), CoreError>> + Send;

    /// A page of the organization's reports, most recent first, together
    /// with the total. Feeds the manager's list and the pending count on the
    /// planning views.
    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        resolution: Option<AssignmentReportResolution>,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<AssignmentReport>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        report: &AssignmentReport,
    ) -> impl Future<Output = Result<AssignmentReport, CoreError>> + Send;

    /// Physical delete — see the migration comment on `assignment_reports`.
    fn delete(
        &mut self,
        id: AssignmentReportId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
