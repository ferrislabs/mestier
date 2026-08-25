use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, ProjectTemplate, ProjectTemplateId, ProjectTemplateTask};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ProjectTemplateRepository: Send {
    fn insert(
        &mut self,
        template: &ProjectTemplate,
    ) -> impl Future<Output = Result<ProjectTemplate, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: ProjectTemplateId,
    ) -> impl Future<Output = Result<Option<ProjectTemplate>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<ProjectTemplate>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        template: &ProjectTemplate,
    ) -> impl Future<Output = Result<ProjectTemplate, CoreError>> + Send;

    /// One method for both directions, like `ProjectRepository::set_archived_at`.
    fn set_archived_at(
        &mut self,
        id: ProjectTemplateId,
        archived_at: Option<DateTime<Utc>>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Full replacement: deletes every shape currently attached to
    /// `template_id` and inserts `tasks` in one go, mirroring how a task's
    /// `PATCH` treats `assignees`. Returns the inserted rows ordered by
    /// `position`.
    fn replace_tasks(
        &mut self,
        template_id: ProjectTemplateId,
        organization_id: OrganizationId,
        tasks: &[ProjectTemplateTask],
    ) -> impl Future<Output = Result<Vec<ProjectTemplateTask>, CoreError>> + Send;

    /// Every task shape of `template_id`, ordered by `position` — the order
    /// instantiation and the builder UI both rely on.
    fn list_tasks(
        &mut self,
        template_id: ProjectTemplateId,
    ) -> impl Future<Output = Result<Vec<ProjectTemplateTask>, CoreError>> + Send;
}
