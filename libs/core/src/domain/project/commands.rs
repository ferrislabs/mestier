use authz::Subject;
use chrono::{DateTime, Utc};

use crate::{CustomerContextId, CustomerId, OrganizationId, ProjectId, QuoteId, QuoteLineId};

#[derive(Debug, Clone)]
pub struct CreateProjectCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
}

/// One task of a confirmed quote-handover plan (#298). `parent_index`
/// refers to another entry's position *within this same list* — the same
/// device `ProjectTemplateTaskShapeCommand` uses, and for the same reason:
/// none of the tasks in the batch have an id yet.
///
/// `starts_at`/`ends_at` follow `CreateTaskCommand`'s own rule: a root
/// (`parent_index: None`) must carry both, a subtask may omit both to
/// inherit its root's window.
#[derive(Debug, Clone)]
pub struct PlannedTaskCommand {
    pub parent_index: Option<usize>,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub blocks_availability: bool,
    pub expenses_cents: i32,
    pub expenses_label: Option<String>,
    /// The quote lines this task accounts for, zero or more. Informational
    /// only: nothing reads a task-to-line mapping back later (a task is a
    /// scheduling unit, a quote line is a commercial one), so this is
    /// validated against the quote's own lines and then dropped rather than
    /// persisted — see `ProjectService::build_planned_tasks`.
    pub quote_line_ids: Vec<QuoteLineId>,
}

/// Turns an accepted quote into a project with the tasks a human confirmed
/// from the plan proposal. One project, a list of tasks each pointing at
/// zero or more quote lines — supply lines and anything else left unmapped
/// simply have no task.
#[derive(Debug, Clone)]
pub struct CreateProjectFromQuoteCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub quote_id: QuoteId,
    pub name: String,
    /// A quote already turned into a project refuses a second one unless
    /// this is explicit — see `validate_quote_plannable`.
    pub force_new: bool,
    pub tasks: Vec<PlannedTaskCommand>,
}

/// Every field is the new complete value, never a delta: clearing the customer
/// means sending `None`, exactly like the `PATCH` contract on tasks treats
/// `assignees` as the full list.
#[derive(Debug, Clone)]
pub struct UpdateProjectCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub id: ProjectId,
    pub name: String,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
}
