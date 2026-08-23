use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    CustomerContextId, CustomerId, OrganizationId, Project, ProjectId, Task, TaskId, TaskStatus,
    domain::{
        project::{
            commands::{CreateProjectCommand, PlannedTaskCommand, UpdateProjectCommand},
            ports::ProjectRepository,
        },
        task::service::normalize_expenses,
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

// ---------------------------------------------------------------------------
// #298 — turning a confirmed quote-handover plan into real tasks. Pure, no
// I/O: every task gets its own freshly generated id up front so a subtask
// can reference its root's id before either is persisted, mirroring
// `ProjectTemplateService`'s own `instantiate_tasks`.
// ---------------------------------------------------------------------------

/// Builds the tasks a confirmed plan produces, attached to `project_id`.
/// Validates the same invariants `TaskService::create_task` enforces per
/// task (a title, a root's own dates, `ends_at` after `starts_at`, the
/// expenses/label pairing) plus the batch-local hierarchy check
/// `ProjectTemplateService::build_shapes` uses for the same reason: none of
/// these tasks have a persisted id yet to check `parent_task_id` against.
pub fn build_planned_tasks(
    commands: &[PlannedTaskCommand],
    project_id: ProjectId,
    organization_id: OrganizationId,
) -> Result<Vec<Task>, CoreError> {
    let now = Utc::now();
    let ids: Vec<TaskId> = commands
        .iter()
        .map(|_| TaskId(generate_uuid_v7()))
        .collect();

    commands
        .iter()
        .enumerate()
        .map(|(index, command)| {
            if command.title.trim().is_empty() {
                return Err(CoreError::Conflict(
                    "a planned task needs a title".to_owned(),
                ));
            }

            let parent_task_id = match command.parent_index {
                Some(parent_index) => {
                    if parent_index == index {
                        return Err(CoreError::Conflict(
                            "a planned task cannot be its own parent".to_owned(),
                        ));
                    }
                    let parent = commands.get(parent_index).ok_or_else(|| {
                        CoreError::Conflict(
                            "a planned task's parent_index does not name another task of the \
                             same plan"
                                .to_owned(),
                        )
                    })?;
                    if parent.parent_index.is_some() {
                        return Err(CoreError::Conflict(
                            "a planned task's parent cannot itself be a subtask".to_owned(),
                        ));
                    }
                    Some(ids[parent_index])
                }
                None => None,
            };

            if command.starts_at.is_some() != command.ends_at.is_some() {
                return Err(CoreError::Conflict(
                    "starts_at and ends_at must be given together".to_owned(),
                ));
            }
            if parent_task_id.is_none() && command.starts_at.is_none() {
                return Err(CoreError::Conflict(
                    "a root task needs its own dates".to_owned(),
                ));
            }
            if let (Some(starts_at), Some(ends_at)) = (command.starts_at, command.ends_at)
                && ends_at <= starts_at
            {
                return Err(CoreError::Conflict(
                    "ends_at must be after starts_at".to_owned(),
                ));
            }

            let (expenses_cents, expenses_label) =
                normalize_expenses(command.expenses_cents, command.expenses_label.clone())?;

            Ok(Task {
                id: ids[index],
                organization_id,
                parent_task_id,
                title: command.title.trim().to_owned(),
                description: command.description.clone(),
                starts_at: command.starts_at,
                ends_at: command.ends_at,
                all_day: command.all_day,
                status: TaskStatus::Planned,
                blocks_availability: command.blocks_availability,
                customer_id: None,
                customer_context_id: None,
                quote_id: None,
                project_id: Some(project_id),
                expenses_cents,
                expenses_label,
                assignments: Vec::new(),
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
        })
        .collect()
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

    // -----------------------------------------------------------------
    // #298 — build_planned_tasks
    // -----------------------------------------------------------------

    fn root_task(title: &str) -> PlannedTaskCommand {
        let now = Utc::now();
        PlannedTaskCommand {
            parent_index: None,
            title: title.to_owned(),
            description: None,
            starts_at: Some(now),
            ends_at: Some(now + chrono::Duration::hours(2)),
            all_day: false,
            blocks_availability: true,
            expenses_cents: 0,
            expenses_label: None,
            quote_line_ids: Vec::new(),
        }
    }

    #[test]
    fn a_root_task_needs_its_own_dates() {
        let mut command = root_task("Terrassement");
        command.starts_at = None;
        command.ends_at = None;

        let err = build_planned_tasks(
            &[command],
            ProjectId(Uuid::new_v4()),
            OrganizationId(Uuid::new_v4()),
        )
        .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn a_subtask_can_inherit_its_root_window() {
        let mut child = root_task("Préparer le matériel");
        child.parent_index = Some(0);
        child.starts_at = None;
        child.ends_at = None;

        let tasks = build_planned_tasks(
            &[root_task("Terrassement"), child],
            ProjectId(Uuid::new_v4()),
            OrganizationId(Uuid::new_v4()),
        )
        .unwrap();

        assert_eq!(tasks[1].parent_task_id, Some(tasks[0].id));
        assert!(tasks[1].starts_at.is_none());
    }

    #[test]
    fn ends_at_before_starts_at_is_refused() {
        let mut command = root_task("Terrassement");
        command.ends_at = command.starts_at;

        let err = build_planned_tasks(
            &[command],
            ProjectId(Uuid::new_v4()),
            OrganizationId(Uuid::new_v4()),
        )
        .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn a_three_level_hierarchy_is_refused() {
        let mut middle = root_task("Étape 2");
        middle.parent_index = Some(0);
        let mut grandchild = root_task("Étape 3");
        grandchild.parent_index = Some(1);

        let err = build_planned_tasks(
            &[root_task("Étape 1"), middle, grandchild],
            ProjectId(Uuid::new_v4()),
            OrganizationId(Uuid::new_v4()),
        )
        .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn every_planned_task_attaches_to_the_project_and_never_the_quote() {
        let project_id = ProjectId(Uuid::new_v4());

        let tasks = build_planned_tasks(
            &[root_task("Terrassement")],
            project_id,
            OrganizationId(Uuid::new_v4()),
        )
        .unwrap();

        assert_eq!(tasks[0].project_id, Some(project_id));
        assert_eq!(tasks[0].quote_id, None);
    }

    #[test]
    fn an_expense_with_no_label_is_refused() {
        let mut command = root_task("Terrassement");
        command.expenses_cents = 4500;

        let err = build_planned_tasks(
            &[command],
            ProjectId(Uuid::new_v4()),
            OrganizationId(Uuid::new_v4()),
        )
        .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }
}
