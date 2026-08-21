use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CustomerId, OrganizationId, Project, ProjectId,
    application::MestierUseCase,
    domain::project::{
        commands::{CreateProjectCommand, UpdateProjectCommand},
        service::ProjectService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(project)]
    pub async fn create_project(
        &self,
        command: CreateProjectCommand,
    ) -> Result<Project, CoreError> {
        let mut service = ProjectService::new(project_repository);
        service.create_project(command).await
    }

    #[transactional(project)]
    pub async fn get_project(&self, id: ProjectId) -> Result<Project, CoreError> {
        let mut service = ProjectService::new(project_repository);
        service.get_project(id).await
    }

    #[transactional(project)]
    pub async fn list_projects(
        &self,
        organization_id: OrganizationId,
        customer_id: Option<CustomerId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Project>, u64), CoreError> {
        let mut service = ProjectService::new(project_repository);
        service
            .list_projects(
                organization_id,
                customer_id,
                include_archived,
                limit,
                offset,
            )
            .await
    }

    #[transactional(project)]
    pub async fn update_project(
        &self,
        command: UpdateProjectCommand,
    ) -> Result<Project, CoreError> {
        let mut service = ProjectService::new(project_repository);
        service.update_project(command).await
    }

    #[transactional(project)]
    pub async fn archive_project(&self, id: ProjectId) -> Result<(), CoreError> {
        let mut service = ProjectService::new(project_repository);
        service.archive_project(id).await
    }

    #[transactional(project)]
    pub async fn restore_project(&self, id: ProjectId) -> Result<(), CoreError> {
        let mut service = ProjectService::new(project_repository);
        service.restore_project(id).await
    }
}
