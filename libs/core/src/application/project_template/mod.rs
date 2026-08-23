use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, Project, ProjectTemplate, ProjectTemplateId, ProjectTemplateTask, Task, Tz,
    application::MestierUseCase,
    domain::{
        organization::ports::OrganizationRepository,
        project_template::{
            commands::{
                CreateProjectTemplateCommand, InstantiateProjectTemplateCommand,
                ReplaceProjectTemplateTasksCommand, UpdateProjectTemplateCommand,
            },
            service::ProjectTemplateService,
        },
    },
};

impl MestierUseCase {
    #[transactional(project_template, project, task)]
    pub async fn create_project_template(
        &self,
        command: CreateProjectTemplateCommand,
    ) -> Result<ProjectTemplate, CoreError> {
        let mut service = ProjectTemplateService::new(
            project_template_repository,
            project_repository,
            task_repository,
        );
        service.create_template(command).await
    }

    #[transactional(project_template, project, task)]
    pub async fn get_project_template(
        &self,
        id: ProjectTemplateId,
    ) -> Result<ProjectTemplate, CoreError> {
        let mut service = ProjectTemplateService::new(
            project_template_repository,
            project_repository,
            task_repository,
        );
        service.get_template(id).await
    }

    #[transactional(project_template, project, task)]
    pub async fn list_project_templates(
        &self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ProjectTemplate>, u64), CoreError> {
        let mut service = ProjectTemplateService::new(
            project_template_repository,
            project_repository,
            task_repository,
        );
        service
            .list_templates(organization_id, include_archived, limit, offset)
            .await
    }

    #[transactional(project_template, project, task)]
    pub async fn list_project_template_tasks(
        &self,
        template_id: ProjectTemplateId,
    ) -> Result<Vec<ProjectTemplateTask>, CoreError> {
        let mut service = ProjectTemplateService::new(
            project_template_repository,
            project_repository,
            task_repository,
        );
        service.list_tasks(template_id).await
    }

    #[transactional(project_template, project, task)]
    pub async fn update_project_template(
        &self,
        command: UpdateProjectTemplateCommand,
    ) -> Result<ProjectTemplate, CoreError> {
        let mut service = ProjectTemplateService::new(
            project_template_repository,
            project_repository,
            task_repository,
        );
        service.update_template(command).await
    }

    #[transactional(project_template, project, task)]
    pub async fn replace_project_template_tasks(
        &self,
        command: ReplaceProjectTemplateTasksCommand,
    ) -> Result<Vec<ProjectTemplateTask>, CoreError> {
        let mut service = ProjectTemplateService::new(
            project_template_repository,
            project_repository,
            task_repository,
        );
        service.replace_tasks(command).await
    }

    #[transactional(project_template, project, task)]
    pub async fn archive_project_template(&self, id: ProjectTemplateId) -> Result<(), CoreError> {
        let mut service = ProjectTemplateService::new(
            project_template_repository,
            project_repository,
            task_repository,
        );
        service.archive_template(id).await
    }

    #[transactional(project_template, project, task)]
    pub async fn restore_project_template(&self, id: ProjectTemplateId) -> Result<(), CoreError> {
        let mut service = ProjectTemplateService::new(
            project_template_repository,
            project_repository,
            task_repository,
        );
        service.restore_template(id).await
    }

    /// Resolves the organization's timezone before handing off to
    /// `ProjectTemplateService::instantiate`, which needs it as a parsed
    /// [`Tz`] to turn every task shape's relative offset into a concrete
    /// window — see that method's own doc comment.
    #[transactional(project_template, project, task, organization)]
    pub async fn instantiate_project_template(
        &self,
        command: InstantiateProjectTemplateCommand,
    ) -> Result<(Project, Vec<Task>), CoreError> {
        let mut organization_repository = organization_repository;
        let timezone = organization_repository
            .find_timezone(command.organization_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let tz: Tz = timezone.parse().map_err(|_| {
            CoreError::Internal(format!("invalid organization timezone `{timezone}`"))
        })?;

        let mut service = ProjectTemplateService::new(
            project_template_repository,
            project_repository,
            task_repository,
        );
        service.instantiate(command, tz).await
    }
}
