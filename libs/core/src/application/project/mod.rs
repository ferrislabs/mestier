use std::collections::HashSet;

use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CustomerId, OrganizationId, Project, ProjectId, Task,
    application::{MestierUseCase, policy},
    domain::{
        project::{
            commands::{CreateProjectCommand, CreateProjectFromQuoteCommand, UpdateProjectCommand},
            ports::ProjectRepository,
            service::{ProjectService, build_planned_tasks},
        },
        quote::ports::QuoteRepository,
        task::ports::TaskRepository,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(project, role, member, authz)]
    pub async fn create_project(
        &self,
        command: CreateProjectCommand,
    ) -> Result<Project, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

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

    #[transactional(project, role, member, authz)]
    pub async fn update_project(
        &self,
        command: UpdateProjectCommand,
    ) -> Result<Project, CoreError> {
        let mut project_repository = project_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        // A bare id derives its organization from the loaded row, never from
        // the command.
        let existing = project_repository
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        let mut service = ProjectService::new(project_repository);
        service.update_project(command).await
    }

    #[transactional(project, role, member, authz)]
    pub async fn archive_project(
        &self,
        actor: authz::Subject,
        id: ProjectId,
    ) -> Result<(), CoreError> {
        let mut project_repository = project_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let existing = project_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        let mut service = ProjectService::new(project_repository);
        service.archive_project(id).await
    }

    #[transactional(project, role, member, authz)]
    pub async fn restore_project(
        &self,
        actor: authz::Subject,
        id: ProjectId,
    ) -> Result<(), CoreError> {
        let mut project_repository = project_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let existing = project_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        let mut service = ProjectService::new(project_repository);
        service.restore_project(id).await
    }

    /// Turns an accepted quote into a project with the tasks a human
    /// confirmed from `GET .../plan-proposal`, in one transaction. The
    /// project carries the quote's customer and the quote itself — that
    /// attachment is what gives `ProjectProfitability::quoted_cents` its
    /// denominator (see #298, and #260 for why `quote_id` lives here and
    /// not on the task).
    #[transactional(project, quote, task, role, member, authz)]
    pub async fn create_project_from_quote(
        &self,
        command: CreateProjectFromQuoteCommand,
    ) -> Result<(Project, Vec<Task>), CoreError> {
        let mut quote_repository = quote_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let quote = quote_repository
            .find_by_id(command.quote_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            quote.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", quote.organization_id.0.to_string()),
        )
        .await?;

        let mut project_repository = project_repository;
        let already_has_project = project_repository
            .exists_for_quote(command.quote_id)
            .await?;
        crate::domain::quote::service::validate_quote_plannable(
            &quote,
            already_has_project,
            command.force_new,
        )?;

        // A planned task may point at zero or more quote lines; the ones it
        // does must actually belong to this quote — a wrong id here is a
        // caller bug worth refusing loudly rather than silently accepting.
        let quote_line_ids: HashSet<_> = quote.lines.iter().map(|line| line.id).collect();
        for task in &command.tasks {
            if task
                .quote_line_ids
                .iter()
                .any(|line_id| !quote_line_ids.contains(line_id))
            {
                return Err(CoreError::Conflict(
                    "a planned task references a quote line that does not belong to this quote"
                        .to_owned(),
                ));
            }
        }

        let mut project_service = ProjectService::new(project_repository);
        let project = project_service
            .create_project(CreateProjectCommand {
                actor: command.actor.clone(),
                organization_id: quote.organization_id,
                name: command.name,
                customer_id: Some(quote.customer_id),
                customer_context_id: Some(quote.customer_context_id),
                quote_id: Some(quote.id),
            })
            .await?;

        let planned_tasks = build_planned_tasks(&command.tasks, project.id, quote.organization_id)?;
        let mut task_repository = task_repository;
        let mut created_tasks = Vec::with_capacity(planned_tasks.len());
        for task in &planned_tasks {
            created_tasks.push(task_repository.insert(task).await?);
        }

        Ok((project, created_tasks))
    }
}
