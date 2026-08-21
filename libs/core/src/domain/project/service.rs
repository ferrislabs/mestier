use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    CustomerContextId, CustomerId, OrganizationId, Project, ProjectId,
    domain::project::{
        commands::{CreateProjectCommand, UpdateProjectCommand},
        ports::ProjectRepository,
    },
};

pub struct ProjectService<R>
where
    R: ProjectRepository,
{
    repo: R,
}

impl<R> ProjectService<R>
where
    R: ProjectRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_project(
        &mut self,
        command: CreateProjectCommand,
    ) -> Result<Project, CoreError> {
        validate_name(&command.name)?;
        validate_customer_pairing(command.customer_id, command.customer_context_id)?;

        let now = Utc::now();
        self.repo
            .insert(&Project {
                id: ProjectId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: command.name,
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                quote_id: command.quote_id,
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_project(&mut self, id: ProjectId) -> Result<Project, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_projects(
        &mut self,
        organization_id: OrganizationId,
        customer_id: Option<CustomerId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Project>, u64), CoreError> {
        self.repo
            .list_by_organization(
                organization_id,
                customer_id,
                include_archived,
                limit,
                offset,
            )
            .await
    }

    pub async fn update_project(
        &mut self,
        command: UpdateProjectCommand,
    ) -> Result<Project, CoreError> {
        validate_name(&command.name)?;
        validate_customer_pairing(command.customer_id, command.customer_context_id)?;

        let mut project = self.get_project(command.id).await?;
        project.name = command.name;
        project.customer_id = command.customer_id;
        project.customer_context_id = command.customer_context_id;
        project.quote_id = command.quote_id;
        project.updated_at = Utc::now();

        self.repo.update(&project).await
    }

    /// Archiving is how a project is retired. There is no hard delete: the
    /// tasks attached to it happened, and their cost is part of a period that
    /// somebody may still be reading.
    pub async fn archive_project(&mut self, id: ProjectId) -> Result<(), CoreError> {
        self.get_project(id).await?;
        self.repo.set_archived_at(id, Some(Utc::now())).await
    }

    pub async fn restore_project(&mut self, id: ProjectId) -> Result<(), CoreError> {
        self.get_project(id).await?;
        self.repo.set_archived_at(id, None).await
    }
}

fn validate_name(name: &str) -> Result<(), CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::Conflict(
            "project name cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

/// A context belongs to a customer, so naming one without the other describes
/// nothing. The reverse is allowed: a customer without a context is a project
/// for a client whose site has not been pinned down yet. Mirrors the `tasks`
/// rule relaxed by the `relax_task_customer_context_pairing` migration, and the
/// `chk_projects_context_requires_customer` constraint enforces the same thing
/// one layer down.
fn validate_customer_pairing(
    customer_id: Option<CustomerId>,
    customer_context_id: Option<CustomerContextId>,
) -> Result<(), CoreError> {
    if customer_context_id.is_some() && customer_id.is_none() {
        return Err(CoreError::Conflict(
            "a project context requires a customer".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::domain::project::ports::MockProjectRepository;

    fn create(name: &str) -> CreateProjectCommand {
        CreateProjectCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            name: name.to_owned(),
            customer_id: None,
            customer_context_id: None,
            quote_id: None,
        }
    }

    #[tokio::test]
    async fn creating_an_internal_project_needs_no_customer() {
        let mut repo = MockProjectRepository::new();
        repo.expect_insert().returning(|project| {
            let project = project.clone();
            Box::pin(async move { Ok(project) })
        });

        let project = ProjectService::new(repo)
            .create_project(create("Réunion hebdo"))
            .await
            .unwrap();

        assert!(project.is_internal());
        assert!(!project.is_archived());
    }

    #[tokio::test]
    async fn a_blank_name_is_refused() {
        let err = ProjectService::new(MockProjectRepository::new())
            .create_project(create("   "))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn a_context_without_a_customer_is_refused() {
        let mut command = create("Chantier");
        command.customer_context_id = Some(CustomerContextId(Uuid::new_v4()));

        let err = ProjectService::new(MockProjectRepository::new())
            .create_project(command)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn a_customer_without_a_context_is_accepted() {
        let mut command = create("Chantier");
        command.customer_id = Some(CustomerId(Uuid::new_v4()));

        let mut repo = MockProjectRepository::new();
        repo.expect_insert().returning(|project| {
            let project = project.clone();
            Box::pin(async move { Ok(project) })
        });

        let project = ProjectService::new(repo)
            .create_project(command)
            .await
            .unwrap();

        assert!(!project.is_internal());
        assert!(project.customer_context_id.is_none());
    }
}
