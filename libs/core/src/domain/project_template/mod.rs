//! Project templates — an ordered set of task shapes, not a project to be
//! copied.
//!
//! An organization that does the same kind of job twenty times a year
//! rebuilds the same task list twenty times, and the twentieth is where a
//! forgotten task becomes an understated project. A template names the shape
//! of the work: titles, offsets relative to a future instantiation date, an
//! optional two-level hierarchy, and expenses. Nothing about who does the
//! job — assignees change every time, and a template that guesses is a
//! template people fight.
//!
//! Instantiation (see [`service::ProjectTemplateService::instantiate`]) is
//! the only place a template touches real data: it takes a name, a start
//! date and optionally a customer and a quote, and produces a real
//! [`crate::Project`] with real [`crate::Task`]s in one transaction.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ProjectTemplateId(pub Uuid);

impl FromStr for ProjectTemplateId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ProjectTemplateId)
    }
}

impl Display for ProjectTemplateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ProjectTemplateTaskId(pub Uuid);

impl FromStr for ProjectTemplateTaskId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ProjectTemplateTaskId)
    }
}

impl Display for ProjectTemplateTaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectTemplate {
    pub id: ProjectTemplateId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    /// Hides the template from pickers without losing what it produced in
    /// the past — same reasoning as `Project::archived_at`.
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProjectTemplate {
    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }
}

/// One task shape belonging to a template. Never carries an assignee, an
/// absolute date, or a status — those only exist once a shape has been
/// turned into a real [`crate::Task`].
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectTemplateTask {
    pub id: ProjectTemplateTaskId,
    pub organization_id: OrganizationId,
    pub template_id: ProjectTemplateId,
    pub title: String,
    pub description: Option<String>,
    /// Relative to whichever date instantiation is given — never an
    /// absolute date, so instantiating a template asks for exactly one
    /// thing.
    pub day_offset: i32,
    /// Minutes since local midnight, mirroring `WorkSlot`. Both absent
    /// together on an all-day shape (see `all_day`), both present
    /// otherwise.
    pub starts_minute: Option<i16>,
    pub ends_minute: Option<i16>,
    /// An all-day shape stays all-day once instantiated, so
    /// `expand_work_slots` costs it from the assignee's slots rather than a
    /// guessed amplitude.
    pub all_day: bool,
    pub blocks_availability: bool,
    pub expenses_cents: i32,
    pub expenses_label: Option<String>,
    /// Points at another task shape of the same template by its
    /// [`Self::position`], not by id — a shape has no id yet to reference
    /// while a template is being built or replaced wholesale. Capped at one
    /// level, the same limit `tasks.parent_task_id` enforces in the domain
    /// (see `ProjectTemplateService::instantiate`).
    pub parent_index: Option<i32>,
    /// The shape's rank within its template, and the value `parent_index`
    /// points at on another row.
    pub position: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template() -> ProjectTemplate {
        let now = Utc::now();

        ProjectTemplate {
            id: ProjectTemplateId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            name: "Pose de terrasse".to_owned(),
            description: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn project_template_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = ProjectTemplateId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn project_template_id_rejects_invalid_uuid() {
        assert!(ProjectTemplateId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn project_template_task_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = ProjectTemplateTaskId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn a_fresh_template_is_not_archived() {
        assert!(!template().is_archived());
    }

    #[test]
    fn an_archived_template_reports_it() {
        let mut template = template();
        template.archived_at = Some(Utc::now());

        assert!(template.is_archived());
    }
}
