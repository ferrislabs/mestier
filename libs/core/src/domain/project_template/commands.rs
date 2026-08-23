use chrono::NaiveDate;

use crate::{CustomerContextId, CustomerId, OrganizationId, ProjectTemplateId, QuoteId};

/// One task shape as carried by a create-with-tasks or replace-tasks
/// request. `parent_index` refers to another entry's position *within this
/// same list* (0-based) — never to a persisted id, since none of the shapes
/// in the batch have one yet.
#[derive(Debug, Clone)]
pub struct ProjectTemplateTaskShapeCommand {
    pub title: String,
    pub description: Option<String>,
    pub day_offset: i32,
    pub starts_minute: Option<i16>,
    pub ends_minute: Option<i16>,
    pub all_day: bool,
    pub blocks_availability: bool,
    pub expenses_cents: i32,
    pub expenses_label: Option<String>,
    pub parent_index: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct CreateProjectTemplateCommand {
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub tasks: Vec<ProjectTemplateTaskShapeCommand>,
}

/// Every field is the new complete value, never a delta — mirrors
/// `UpdateProjectCommand`. Tasks are replaced through their own command
/// ([`ReplaceProjectTemplateTasksCommand`]), never here, so a caller can
/// rename a template without resending every task shape.
#[derive(Debug, Clone)]
pub struct UpdateProjectTemplateCommand {
    pub id: ProjectTemplateId,
    pub name: String,
    pub description: Option<String>,
}

/// The complete replacement set of a template's task shapes, mirroring the
/// `assignees`/`label_ids` convention on `PatchTaskCommand`: always the full
/// list, never a delta.
#[derive(Debug, Clone)]
pub struct ReplaceProjectTemplateTasksCommand {
    pub template_id: ProjectTemplateId,
    pub tasks: Vec<ProjectTemplateTaskShapeCommand>,
}

/// Turns a template into a real project with real tasks. `start_date`
/// resolves every task shape's `day_offset` against the organization's
/// timezone — see `ProjectTemplateService::instantiate`.
#[derive(Debug, Clone)]
pub struct InstantiateProjectTemplateCommand {
    pub template_id: ProjectTemplateId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub start_date: NaiveDate,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
}
