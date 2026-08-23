use crate::{
    MemberId, TaskAssignmentId, domain::assignment_report::AssignmentReportId,
    domain::assignment_report::AssignmentReportResolution,
};

/// `reported_by` is resolved by the caller from the authenticated identity,
/// never carried by an HTTP request payload — mirrors
/// `CreateTaskCommentCommand::author_user_id`. A client that could name its
/// own reporter could file on someone else's assignment.
#[derive(Debug, Clone)]
pub struct ReportAssignmentCommand {
    pub task_assignment_id: TaskAssignmentId,
    pub reported_by: MemberId,
    pub reported_minutes: u32,
    pub comment: Option<String>,
}

/// Amends a still-pending report — the worker changing their mind, not a
/// second opinion. `acting_member_id` is checked against the report's own
/// `reported_by` by
/// [`crate::domain::assignment_report::service::AssignmentReportService::amend_report`].
#[derive(Debug, Clone)]
pub struct AmendAssignmentReportCommand {
    pub id: AssignmentReportId,
    pub acting_member_id: MemberId,
    pub reported_minutes: u32,
    pub comment: Option<String>,
}

/// Withdraws a still-pending report. Physical deletion — see the migration
/// comment on `assignment_reports`.
#[derive(Debug, Clone)]
pub struct WithdrawAssignmentReportCommand {
    pub id: AssignmentReportId,
    pub acting_member_id: MemberId,
}

/// The manager's arbitration. `resolution` must be `Applied` or `Dismissed`
/// — `Pending` is refused by
/// [`crate::domain::assignment_report::service::AssignmentReportService::resolve_report`],
/// since it is not something a manager decides *into*.
#[derive(Debug, Clone)]
pub struct ResolveAssignmentReportCommand {
    pub id: AssignmentReportId,
    pub resolved_by: MemberId,
    pub resolution: AssignmentReportResolution,
    pub resolution_note: Option<String>,
}
