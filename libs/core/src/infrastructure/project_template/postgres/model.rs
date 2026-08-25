use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    OrganizationId, ProjectTemplate, ProjectTemplateId, ProjectTemplateTask, ProjectTemplateTaskId,
};

#[derive(Debug, Clone)]
pub struct ProjectTemplateRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProjectTemplateRow> for ProjectTemplate {
    fn from(row: ProjectTemplateRow) -> Self {
        Self {
            id: ProjectTemplateId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            description: row.description,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectTemplateTaskRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub template_id: Uuid,
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
    pub position: i32,
}

impl From<ProjectTemplateTaskRow> for ProjectTemplateTask {
    fn from(row: ProjectTemplateTaskRow) -> Self {
        Self {
            id: ProjectTemplateTaskId(row.id),
            organization_id: OrganizationId(row.org_id),
            template_id: ProjectTemplateId(row.template_id),
            title: row.title,
            description: row.description,
            day_offset: row.day_offset,
            starts_minute: row.starts_minute,
            ends_minute: row.ends_minute,
            all_day: row.all_day,
            blocks_availability: row.blocks_availability,
            expenses_cents: row.expenses_cents,
            expenses_label: row.expenses_label,
            parent_index: row.parent_index,
            position: row.position,
        }
    }
}
