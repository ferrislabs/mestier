use chrono::{DateTime, LocalResult, NaiveDate, TimeZone, Utc};
use common::{CoreError, generate_uuid_v7};

use crate::{
    OrganizationId, Project, ProjectId, Task, TaskId, TaskStatus, Tz,
    domain::{
        project::ports::ProjectRepository,
        project_template::{
            ProjectTemplate, ProjectTemplateId, ProjectTemplateTask,
            commands::{
                CreateProjectTemplateCommand, InstantiateProjectTemplateCommand,
                ProjectTemplateTaskShapeCommand, ReplaceProjectTemplateTasksCommand,
                UpdateProjectTemplateCommand,
            },
            ports::ProjectTemplateRepository,
        },
        task::{ports::TaskRepository, service::normalize_expenses},
    },
};

// ---------------------------------------------------------------------------
// Pure functions — no I/O, exercised directly by the unit tests below.
// ---------------------------------------------------------------------------

/// Validates one task shape in isolation: title, expenses pairing, and the
/// `all_day` / minute-of-day equivalence. Does not know about its siblings
/// — hierarchy validation (`parent_index`) needs the whole batch, so it
/// lives in [`build_shapes`] instead.
fn validate_shape(
    shape: &ProjectTemplateTaskShapeCommand,
) -> Result<(i32, Option<String>), CoreError> {
    if shape.title.trim().is_empty() {
        return Err(CoreError::Conflict(
            "a template task needs a title".to_owned(),
        ));
    }

    if shape.all_day {
        if shape.starts_minute.is_some() || shape.ends_minute.is_some() {
            return Err(CoreError::Conflict(
                "an all-day template task cannot carry a start or end time".to_owned(),
            ));
        }
    } else if shape.starts_minute.is_none() || shape.ends_minute.is_none() {
        return Err(CoreError::Conflict(
            "a template task needs both a start and an end time, unless it is all-day".to_owned(),
        ));
    }

    if let (Some(starts_minute), Some(ends_minute)) = (shape.starts_minute, shape.ends_minute) {
        let in_range = |minute: i16| (0..=1440).contains(&minute);
        if !in_range(starts_minute) || !in_range(ends_minute) || ends_minute <= starts_minute {
            return Err(CoreError::Conflict(
                "a template task's start and end time must fall within one day, start before \
                 end"
                .to_owned(),
            ));
        }
    }

    normalize_expenses(shape.expenses_cents, shape.expenses_label.clone())
}

/// Builds the ordered set of [`ProjectTemplateTask`]s a create/replace
/// command produces, assigning `position` from array order and validating
/// the hierarchy: a shape's `parent_index` must name another, distinct
/// shape of the same batch that is itself a root (the same two-level cap
/// `tasks.parent_task_id` enforces, applied here since there is no
/// persisted parent to look up yet).
fn build_shapes(
    template_id: ProjectTemplateId,
    organization_id: OrganizationId,
    commands: &[ProjectTemplateTaskShapeCommand],
) -> Result<Vec<ProjectTemplateTask>, CoreError> {
    for (index, shape) in commands.iter().enumerate() {
        if let Some(parent_index) = shape.parent_index {
            let parent_index = usize::try_from(parent_index).ok().filter(|i| *i != index);
            let parent = parent_index.and_then(|i| commands.get(i));
            let Some(parent) = parent else {
                return Err(CoreError::Conflict(
                    "a template task's parent_index does not name another task of the same \
                     template"
                        .to_owned(),
                ));
            };
            if parent.parent_index.is_some() {
                return Err(CoreError::Conflict(
                    "a template task's parent cannot itself be a subtask".to_owned(),
                ));
            }
        }
    }

    commands
        .iter()
        .enumerate()
        .map(|(index, shape)| {
            let (expenses_cents, expenses_label) = validate_shape(shape)?;

            Ok(ProjectTemplateTask {
                id: crate::domain::project_template::ProjectTemplateTaskId(generate_uuid_v7()),
                organization_id,
                template_id,
                title: shape.title.clone(),
                description: shape.description.clone(),
                day_offset: shape.day_offset,
                starts_minute: shape.starts_minute,
                ends_minute: shape.ends_minute,
                all_day: shape.all_day,
                blocks_availability: shape.blocks_availability,
                expenses_cents,
                expenses_label,
                parent_index: shape.parent_index,
                position: index as i32,
            })
        })
        .collect()
}

fn validate_name(name: &str) -> Result<(), CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::Conflict(
            "project template name cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

/// Converts a local calendar minute-of-day back to a UTC instant, via `tz`.
/// Duplicated from `domain::planning::service::local_minute_to_utc` rather
/// than shared: the two live in unrelated aggregates, and the function is
/// small enough that copying it costs less than the coupling a shared
/// module would add (see the ambiguous/gap handling note there — the same
/// conservative choices apply here).
fn local_minute_to_utc(date: NaiveDate, minute: i32, tz: Tz) -> DateTime<Utc> {
    let naive = date
        .and_hms_opt(0, 0, 0)
        .expect("midnight is always a valid time")
        + chrono::Duration::minutes(minute as i64);

    match tz.from_local_datetime(&naive) {
        LocalResult::Single(dt) => dt.with_timezone(&Utc),
        LocalResult::Ambiguous(earliest, _latest) => earliest.with_timezone(&Utc),
        LocalResult::None => Utc.from_utc_datetime(&naive),
    }
}

/// Resolves one shape's window against `start_date` and `tz`. An all-day
/// shape becomes the `[00:00, 24:00)` window of its local day — the same
/// convention `work_orders`' own migration documents for `all_day` tasks.
fn resolve_shape_window(
    shape: &ProjectTemplateTask,
    start_date: NaiveDate,
    tz: Tz,
) -> Result<(DateTime<Utc>, DateTime<Utc>), CoreError> {
    let date = start_date
        .checked_add_signed(chrono::Duration::days(shape.day_offset as i64))
        .ok_or_else(|| {
            CoreError::Conflict("a template task's day_offset is out of range".to_owned())
        })?;

    if shape.all_day {
        let next_date = date.succ_opt().ok_or_else(|| {
            CoreError::Conflict("a template task's day_offset is out of range".to_owned())
        })?;
        return Ok((
            local_minute_to_utc(date, 0, tz),
            local_minute_to_utc(next_date, 0, tz),
        ));
    }

    let starts_minute = shape.starts_minute.expect(
        "validated at creation/replace time: a non-all-day shape always carries both minutes",
    );
    let ends_minute = shape.ends_minute.expect(
        "validated at creation/replace time: a non-all-day shape always carries both minutes",
    );

    Ok((
        local_minute_to_utc(date, starts_minute as i32, tz),
        local_minute_to_utc(date, ends_minute as i32, tz),
    ))
}

/// Turns an ordered set of task shapes into real [`Task`]s attached to
/// `project_id`. Every shape gets its own freshly generated id up front, so
/// a subtask (`parent_index.is_some()`) can reference its parent's id even
/// though the parent has not been persisted yet.
fn instantiate_tasks(
    shapes: &[ProjectTemplateTask],
    project_id: ProjectId,
    organization_id: OrganizationId,
    start_date: NaiveDate,
    tz: Tz,
) -> Result<Vec<Task>, CoreError> {
    let now = Utc::now();
    let ids: Vec<TaskId> = shapes.iter().map(|_| TaskId(generate_uuid_v7())).collect();

    shapes
        .iter()
        .enumerate()
        .map(|(index, shape)| {
            let parent_task_id = match shape.parent_index {
                Some(parent_index) => {
                    let parent_index = usize::try_from(parent_index).map_err(|_| {
                        CoreError::Conflict(
                            "a template task's parent_index cannot be negative".to_owned(),
                        )
                    })?;
                    let parent_shape = shapes.get(parent_index).ok_or_else(|| {
                        CoreError::Conflict(
                            "a template task's parent_index does not name another task of the \
                             same template"
                                .to_owned(),
                        )
                    })?;
                    if parent_shape.parent_index.is_some() {
                        return Err(CoreError::Conflict(
                            "a template task's parent cannot itself be a subtask".to_owned(),
                        ));
                    }
                    Some(ids[parent_index])
                }
                None => None,
            };

            let (starts_at, ends_at) = resolve_shape_window(shape, start_date, tz)?;

            Ok(Task {
                id: ids[index],
                organization_id,
                parent_task_id,
                title: shape.title.clone(),
                description: shape.description.clone(),
                starts_at: Some(starts_at),
                ends_at: Some(ends_at),
                all_day: shape.all_day,
                status: TaskStatus::Planned,
                blocks_availability: shape.blocks_availability,
                customer_id: None,
                customer_context_id: None,
                quote_id: None,
                project_id: Some(project_id),
                expenses_cents: shape.expenses_cents,
                expenses_label: shape.expenses_label.clone(),
                assignments: Vec::new(),
                recurrence_id: None,
                occurrence_date: None,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// ProjectTemplateService — I/O-bound orchestration.
// ---------------------------------------------------------------------------

/// Composes the template repository with `project` and `task` — the two
/// aggregates instantiation produces rows in — mirroring how
/// `PlanningService` composes several read-side repositories. Cross-aggregate
/// orchestration belongs here, not in the thin `#[transactional]`
/// application layer.
pub struct ProjectTemplateService<PTR, PR, TR>
where
    PTR: ProjectTemplateRepository,
    PR: ProjectRepository,
    TR: TaskRepository,
{
    template_repository: PTR,
    project_repository: PR,
    task_repository: TR,
}

impl<PTR, PR, TR> ProjectTemplateService<PTR, PR, TR>
where
    PTR: ProjectTemplateRepository,
    PR: ProjectRepository,
    TR: TaskRepository,
{
    pub fn new(template_repository: PTR, project_repository: PR, task_repository: TR) -> Self {
        Self {
            template_repository,
            project_repository,
            task_repository,
        }
    }

    pub async fn create_template(
        &mut self,
        command: CreateProjectTemplateCommand,
    ) -> Result<ProjectTemplate, CoreError> {
        validate_name(&command.name)?;

        let now = Utc::now();
        let template = self
            .template_repository
            .insert(&ProjectTemplate {
                id: ProjectTemplateId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: command.name,
                description: command.description,
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        if !command.tasks.is_empty() {
            let shapes = build_shapes(template.id, command.organization_id, &command.tasks)?;
            self.template_repository
                .replace_tasks(template.id, command.organization_id, &shapes)
                .await?;
        }

        Ok(template)
    }

    pub async fn get_template(
        &mut self,
        id: ProjectTemplateId,
    ) -> Result<ProjectTemplate, CoreError> {
        self.template_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_templates(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ProjectTemplate>, u64), CoreError> {
        self.template_repository
            .list_by_organization(organization_id, include_archived, limit, offset)
            .await
    }

    pub async fn list_tasks(
        &mut self,
        template_id: ProjectTemplateId,
    ) -> Result<Vec<ProjectTemplateTask>, CoreError> {
        self.template_repository.list_tasks(template_id).await
    }

    pub async fn update_template(
        &mut self,
        command: UpdateProjectTemplateCommand,
    ) -> Result<ProjectTemplate, CoreError> {
        validate_name(&command.name)?;

        let mut template = self.get_template(command.id).await?;
        template.name = command.name;
        template.description = command.description;
        template.updated_at = Utc::now();

        self.template_repository.update(&template).await
    }

    pub async fn replace_tasks(
        &mut self,
        command: ReplaceProjectTemplateTasksCommand,
    ) -> Result<Vec<ProjectTemplateTask>, CoreError> {
        let template = self.get_template(command.template_id).await?;
        let shapes = build_shapes(template.id, template.organization_id, &command.tasks)?;

        self.template_repository
            .replace_tasks(template.id, template.organization_id, &shapes)
            .await
    }

    pub async fn archive_template(&mut self, id: ProjectTemplateId) -> Result<(), CoreError> {
        self.get_template(id).await?;
        self.template_repository
            .set_archived_at(id, Some(Utc::now()))
            .await
    }

    pub async fn restore_template(&mut self, id: ProjectTemplateId) -> Result<(), CoreError> {
        self.get_template(id).await?;
        self.template_repository.set_archived_at(id, None).await
    }

    /// Takes a template, a name, a start date and optionally a customer and
    /// a quote, and produces a project with its tasks in one transaction
    /// (the caller wraps this call in one, via `#[transactional]`). Offsets
    /// resolve against `tz` — the organization's own timezone, resolved by
    /// the application layer before calling this.
    pub async fn instantiate(
        &mut self,
        command: InstantiateProjectTemplateCommand,
        tz: Tz,
    ) -> Result<(Project, Vec<Task>), CoreError> {
        let template = self.get_template(command.template_id).await?;
        if template.organization_id != command.organization_id {
            return Err(CoreError::NotFound);
        }
        if template.is_archived() {
            return Err(CoreError::Conflict(
                "an archived template cannot be instantiated".to_owned(),
            ));
        }
        validate_name(&command.name)?;

        let shapes = self
            .template_repository
            .list_tasks(command.template_id)
            .await?;

        let now = Utc::now();
        let project = self
            .project_repository
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
            .await?;

        let tasks = instantiate_tasks(
            &shapes,
            project.id,
            command.organization_id,
            command.start_date,
            tz,
        )?;

        let mut created_tasks = Vec::with_capacity(tasks.len());
        for task in &tasks {
            created_tasks.push(self.task_repository.insert(task).await?);
        }

        Ok((project, created_tasks))
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::domain::{
        project::ports::MockProjectRepository,
        project_template::ports::MockProjectTemplateRepository, task::ports::MockTaskRepository,
    };

    fn organization_id() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn shape(day_offset: i32) -> ProjectTemplateTaskShapeCommand {
        ProjectTemplateTaskShapeCommand {
            title: "Préparer le chantier".to_owned(),
            description: None,
            day_offset,
            starts_minute: Some(480),
            ends_minute: Some(720),
            all_day: false,
            blocks_availability: true,
            expenses_cents: 0,
            expenses_label: None,
            parent_index: None,
        }
    }

    #[tokio::test]
    async fn creating_a_template_needs_a_name() {
        let mut service = ProjectTemplateService::new(
            MockProjectTemplateRepository::new(),
            MockProjectRepository::new(),
            MockTaskRepository::new(),
        );

        let err = service
            .create_template(CreateProjectTemplateCommand {
                organization_id: organization_id(),
                name: "   ".to_owned(),
                description: None,
                tasks: Vec::new(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn a_template_task_needs_a_title() {
        let mut template_repository = MockProjectTemplateRepository::new();
        template_repository.expect_insert().returning(|template| {
            let template = template.clone();
            Box::pin(async move { Ok(template) })
        });

        let mut service = ProjectTemplateService::new(
            template_repository,
            MockProjectRepository::new(),
            MockTaskRepository::new(),
        );

        let mut task = shape(0);
        task.title = "  ".to_owned();

        let err = service
            .create_template(CreateProjectTemplateCommand {
                organization_id: organization_id(),
                name: "Terrasse".to_owned(),
                description: None,
                tasks: vec![task],
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn a_non_all_day_shape_needs_both_times() {
        let mut task = shape(0);
        task.ends_minute = None;

        let err = validate_shape(&task).unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn an_all_day_shape_cannot_carry_a_time() {
        let mut task = shape(0);
        task.all_day = true;

        let err = validate_shape(&task).unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn a_shape_expense_with_no_label_is_refused() {
        let mut task = shape(0);
        task.expenses_cents = 500;

        let err = validate_shape(&task).unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn build_shapes_assigns_position_from_array_order() {
        let template_id = ProjectTemplateId(Uuid::new_v4());
        let organization_id = organization_id();

        let shapes = build_shapes(
            template_id,
            organization_id,
            &[shape(0), shape(1), shape(2)],
        )
        .unwrap();

        assert_eq!(shapes[0].position, 0);
        assert_eq!(shapes[1].position, 1);
        assert_eq!(shapes[2].position, 2);
    }

    #[test]
    fn a_subtask_can_reference_a_root_by_index() {
        let mut child = shape(1);
        child.parent_index = Some(0);

        let shapes = build_shapes(
            ProjectTemplateId(Uuid::new_v4()),
            organization_id(),
            &[shape(0), child],
        )
        .unwrap();

        assert_eq!(shapes[1].parent_index, Some(0));
    }

    #[test]
    fn a_subtask_cannot_itself_be_a_parent() {
        let mut middle = shape(1);
        middle.parent_index = Some(0);
        let mut grandchild = shape(2);
        grandchild.parent_index = Some(1);

        let err = build_shapes(
            ProjectTemplateId(Uuid::new_v4()),
            organization_id(),
            &[shape(0), middle, grandchild],
        )
        .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn a_parent_index_naming_nothing_is_refused() {
        let mut child = shape(1);
        child.parent_index = Some(9);

        let err = build_shapes(
            ProjectTemplateId(Uuid::new_v4()),
            organization_id(),
            &[shape(0), child],
        )
        .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn a_shape_cannot_be_its_own_parent() {
        let mut root = shape(0);
        root.parent_index = Some(0);

        let err = build_shapes(
            ProjectTemplateId(Uuid::new_v4()),
            organization_id(),
            &[root],
        )
        .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn an_all_day_shape_resolves_to_the_full_local_day() {
        let template_task = ProjectTemplateTask {
            id: crate::domain::project_template::ProjectTemplateTaskId(Uuid::new_v4()),
            organization_id: organization_id(),
            template_id: ProjectTemplateId(Uuid::new_v4()),
            title: "Livraison matériel".to_owned(),
            description: None,
            day_offset: 2,
            starts_minute: None,
            ends_minute: None,
            all_day: true,
            blocks_availability: true,
            expenses_cents: 0,
            expenses_label: None,
            parent_index: None,
            position: 0,
        };

        let start_date = NaiveDate::from_ymd_opt(2026, 8, 24).unwrap();
        let tz: Tz = "Europe/Paris".parse().unwrap();

        let (starts_at, ends_at) = resolve_shape_window(&template_task, start_date, tz).unwrap();

        let expected_date = NaiveDate::from_ymd_opt(2026, 8, 26).unwrap();
        assert_eq!(starts_at.with_timezone(&tz).date_naive(), expected_date);
        assert_eq!(
            ends_at.with_timezone(&tz).date_naive(),
            expected_date.succ_opt().unwrap()
        );
    }

    #[test]
    fn instantiate_tasks_wires_a_subtask_to_its_freshly_generated_parent_id() {
        let organization_id = organization_id();
        let project_id = ProjectId(Uuid::new_v4());
        let template_id = ProjectTemplateId(Uuid::new_v4());
        let start_date = NaiveDate::from_ymd_opt(2026, 8, 24).unwrap();
        let tz: Tz = "Europe/Paris".parse().unwrap();

        let root = ProjectTemplateTask {
            id: crate::domain::project_template::ProjectTemplateTaskId(Uuid::new_v4()),
            organization_id,
            template_id,
            title: "Chantier".to_owned(),
            description: None,
            day_offset: 0,
            starts_minute: Some(480),
            ends_minute: Some(1020),
            all_day: false,
            blocks_availability: true,
            expenses_cents: 0,
            expenses_label: None,
            parent_index: None,
            position: 0,
        };
        let mut child = root.clone();
        child.id = crate::domain::project_template::ProjectTemplateTaskId(Uuid::new_v4());
        child.title = "Préparer le matériel".to_owned();
        child.parent_index = Some(0);
        child.position = 1;

        let tasks =
            instantiate_tasks(&[root, child], project_id, organization_id, start_date, tz).unwrap();

        assert_eq!(tasks[1].parent_task_id, Some(tasks[0].id));
        assert_eq!(tasks[0].project_id, Some(project_id));
        assert_eq!(tasks[1].project_id, Some(project_id));
    }
}
