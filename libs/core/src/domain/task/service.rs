use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};

use crate::{
    CustomerContextId, CustomerId, Employee, EmployeeId, OrganizationId, Task, TimeRange,
    domain::{
        employee::ports::EmployeeRepository,
        member::ports::MemberRepository,
        task::{
            TaskAssignment, TaskAssignmentId, TaskId,
            commands::{CreateTaskCommand, PatchTaskCommand},
            ports::TaskRepository,
        },
        user::ports::UserRepository,
    },
};

// ---------------------------------------------------------------------------
// The two pure functions this workstream is built around — no I/O, disproportionate
// test coverage relative to their size (see the planning module design doc,
// which puts them on par with `expand_work_slots` and `detect_conflicts`).
// ---------------------------------------------------------------------------

/// Resolves a task's effective window: its own if it carries one, its
/// parent's otherwise.
///
/// A root always carries its own dates (enforced by
/// [`TaskService::create_task`] and the `chk_tasks_root_has_dates`
/// constraint), so `parent` only matters for a subtask that omitted
/// `starts_at`/`ends_at` — which means "inherit". Resolving here, at read
/// time, rather than copying the parent's dates down at creation, avoids a
/// duplicate that would silently drift the moment the parent is rescheduled
/// (see the planning module design doc).
pub fn resolve_task_window(task: &Task, parent: Option<&Task>) -> TimeRange {
    if let (Some(starts_at), Some(ends_at)) = (task.starts_at, task.ends_at) {
        return TimeRange { starts_at, ends_at };
    }

    let parent = parent.expect(
        "a task without its own dates must have a parent — enforced at creation by \
         `TaskService::create_task` and the `chk_tasks_root_has_dates` constraint",
    );

    TimeRange {
        starts_at: parent.starts_at.expect(
            "a task usable as a parent is always a root, and a root always carries its own \
             dates — enforced by `chk_tasks_root_has_dates`",
        ),
        ends_at: parent
            .ends_at
            .expect("see the sibling `starts_at` panic: a root carries both or neither"),
    }
}

/// Rejects a parent that itself has a parent. The two-level hierarchy limit
/// lives here, in the domain, not in the schema — lifting it later is a
/// validation-rule change, not a migration (see the planning module design
/// doc).
pub fn validate_parent_depth(parent: Option<&Task>) -> Result<(), CoreError> {
    match parent {
        Some(parent) if parent.parent_task_id.is_some() => Err(CoreError::Conflict(
            "a subtask's parent cannot itself be a subtask".to_owned(),
        )),
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// TaskService — I/O-bound orchestration.
// ---------------------------------------------------------------------------

/// Orchestrates tasks together with the employee, user and member
/// repositories it needs to resolve `PATCH`'s `assignees` — in particular
/// the on-the-fly employee creation for a `member` assignee who does not
/// have an employee record yet, and the parent lookup `validate_parent_depth`
/// needs. This mirrors how `OrganizationService` composes
/// `role`/`member`/`user` repositories directly: cross-aggregate
/// orchestration lives in the domain service that owns the use case, never
/// in the thin `#[transactional]` application layer.
pub struct TaskService<TR, ER, UR, MR>
where
    TR: TaskRepository,
    ER: EmployeeRepository,
    UR: UserRepository,
    MR: MemberRepository,
{
    task_repository: TR,
    employee_repository: ER,
    user_repository: UR,
    member_repository: MR,
}

impl<TR, ER, UR, MR> TaskService<TR, ER, UR, MR>
where
    TR: TaskRepository,
    ER: EmployeeRepository,
    UR: UserRepository,
    MR: MemberRepository,
{
    pub fn new(
        task_repository: TR,
        employee_repository: ER,
        user_repository: UR,
        member_repository: MR,
    ) -> Self {
        Self {
            task_repository,
            employee_repository,
            user_repository,
            member_repository,
        }
    }

    pub async fn create_task(&mut self, command: CreateTaskCommand) -> Result<Task, CoreError> {
        validate_title(&command.title)?;
        validate_text_field("task description", &command.description)?;
        validate_customer_pairing(command.customer_id, command.customer_context_id)?;

        if let Some(parent_id) = command.parent_task_id {
            let parent = self
                .task_repository
                .find_by_id(parent_id)
                .await?
                .ok_or(CoreError::NotFound)?;
            if parent.organization_id != command.organization_id {
                return Err(CoreError::NotFound);
            }
            validate_parent_depth(Some(&parent))?;
        }
        validate_task_dates(
            command.parent_task_id.is_some(),
            command.starts_at,
            command.ends_at,
        )?;

        let now = Utc::now();
        self.task_repository
            .insert(&Task {
                id: TaskId(generate_uuid_v7()),
                organization_id: command.organization_id,
                parent_task_id: command.parent_task_id,
                title: command.title,
                description: command.description,
                starts_at: command.starts_at,
                ends_at: command.ends_at,
                all_day: command.all_day,
                status: crate::TaskStatus::Planned,
                blocks_availability: command.blocks_availability,
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                quote_id: command.quote_id,
                assignments: Vec::new(),
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_task(&mut self, id: TaskId) -> Result<Task, CoreError> {
        self.task_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    /// Lists a page of `organization_id`'s tasks — every root when
    /// `parent_task_id` is `None`, or the children of a specific task
    /// otherwise — together with each returned task's own child count,
    /// fetched in one grouped query rather than one per task (see
    /// `TaskRepository::count_children`'s N+1 warning).
    pub async fn list_tasks(
        &mut self,
        organization_id: OrganizationId,
        parent_task_id: Option<TaskId>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Task>, HashMap<TaskId, i64>, u64), CoreError> {
        let (tasks, total) = self
            .task_repository
            .list_by_organization(organization_id, parent_task_id, limit, offset)
            .await?;
        let ids: Vec<TaskId> = tasks.iter().map(|task| task.id).collect();
        let child_counts = self.task_repository.count_children(&ids).await?;

        Ok((tasks, child_counts, total))
    }

    /// Applies a `PATCH`: reparenting, reschedule, status/title/description
    /// edits, the `blocks_availability` flag, and a full assignee
    /// replacement, all against the single `Task` loaded at the top — the
    /// caller wraps this in one transaction (see
    /// `#[transactional(task, employee, user, member)]` on
    /// `MestierUseCase::patch_task`), so either every write here lands or
    /// none does.
    ///
    /// Returns the updated task together with the employee records created
    /// on the fly for `member` assignees who had none yet.
    pub async fn patch_task(
        &mut self,
        command: PatchTaskCommand,
    ) -> Result<(Task, Vec<Employee>), CoreError> {
        let mut task = self.get_task(command.id).await?;

        if let Some(parent_choice) = command.parent_task_id {
            match parent_choice {
                Some(parent_id) => {
                    if parent_id == task.id {
                        return Err(CoreError::Conflict(
                            "a task cannot be its own parent".to_owned(),
                        ));
                    }
                    let parent = self
                        .task_repository
                        .find_by_id(parent_id)
                        .await?
                        .ok_or(CoreError::NotFound)?;
                    if parent.organization_id != task.organization_id {
                        return Err(CoreError::NotFound);
                    }
                    validate_parent_depth(Some(&parent))?;
                    task.parent_task_id = Some(parent_id);
                }
                None => task.parent_task_id = None,
            }
        }

        let title = command.title.unwrap_or_else(|| task.title.clone());
        validate_title(&title)?;
        let description = command
            .description
            .unwrap_or_else(|| task.description.clone());
        validate_text_field("task description", &description)?;

        let starts_at = command.starts_at.unwrap_or(task.starts_at);
        let ends_at = command.ends_at.unwrap_or(task.ends_at);
        validate_task_dates(task.parent_task_id.is_some(), starts_at, ends_at)?;

        task.title = title;
        task.description = description;
        task.starts_at = starts_at;
        task.ends_at = ends_at;
        if let Some(all_day) = command.all_day {
            task.all_day = all_day;
        }
        if let Some(status) = command.status {
            task.status = status;
        }
        if let Some(blocks_availability) = command.blocks_availability {
            task.blocks_availability = blocks_availability;
        }
        task.updated_at = Utc::now();

        let created_employees = if let Some(assignees) = command.assignees {
            let (assignments, created_employees) =
                self.resolve_assignments(&task, assignees).await?;
            task.assignments = assignments;
            created_employees
        } else {
            Vec::new()
        };

        let updated = self.task_repository.update(&task).await?;
        Ok((updated, created_employees))
    }

    /// Resolves each `AssigneeRef` to a concrete, deduplicated `employee_id`
    /// list, provisioning an employee record on the fly for `member`
    /// assignees who don't have one yet (`hourly_rate_cents` stays `NULL`
    /// — never defaulted to `0`, which would read as "genuinely free").
    async fn resolve_assignments(
        &mut self,
        task: &Task,
        assignees: Vec<crate::AssigneeRef>,
    ) -> Result<(Vec<TaskAssignment>, Vec<Employee>), CoreError> {
        let mut created_employees = Vec::new();
        let mut seen = HashSet::new();
        let mut employee_ids = Vec::with_capacity(assignees.len());

        for assignee in assignees {
            let employee_id = self
                .resolve_assignee(task, assignee, &mut created_employees)
                .await?;

            if seen.insert(employee_id) {
                employee_ids.push(employee_id);
            }
        }

        let now = Utc::now();
        let assignments = employee_ids
            .into_iter()
            .map(|employee_id| TaskAssignment {
                id: TaskAssignmentId(generate_uuid_v7()),
                organization_id: task.organization_id,
                task_id: task.id,
                employee_id,
                created_at: now,
            })
            .collect();

        Ok((assignments, created_employees))
    }

    async fn resolve_assignee(
        &mut self,
        task: &Task,
        assignee: crate::AssigneeRef,
        created_employees: &mut Vec<Employee>,
    ) -> Result<EmployeeId, CoreError> {
        match assignee {
            crate::AssigneeRef::Employee(employee_id) => {
                let employee = self
                    .employee_repository
                    .find_by_id(employee_id)
                    .await?
                    .ok_or(CoreError::NotFound)?;

                if employee.organization_id != task.organization_id {
                    return Err(CoreError::NotFound);
                }

                Ok(employee_id)
            }
            crate::AssigneeRef::Member(user_id) => {
                self.member_repository
                    .find_by_org_and_user(task.organization_id, user_id)
                    .await?
                    .ok_or(CoreError::NotFound)?;

                if let Some(existing) = self
                    .employee_repository
                    .find_by_user_id(task.organization_id, user_id)
                    .await?
                {
                    return Ok(existing.id);
                }

                let user = self
                    .user_repository
                    .find_by_id(user_id)
                    .await?
                    .ok_or(CoreError::NotFound)?;

                let now = Utc::now();
                let created = self
                    .employee_repository
                    .insert(&Employee {
                        id: EmployeeId(generate_uuid_v7()),
                        organization_id: task.organization_id,
                        user_id: Some(user_id),
                        // The account's `display_name` is a single free-text
                        // field, exactly like the pre-split `employees.name`
                        // was — it cannot be reliably split into a first and
                        // last name (see the `split_employee_name`
                        // migration), so it becomes `last_name` and
                        // `first_name` is left unset, same as a backfilled row.
                        last_name: user.name,
                        first_name: None,
                        // Never `Some(0)`: an on-the-fly record has no rate
                        // *yet*, which is not the same as "free".
                        hourly_rate_cents: None,
                        weekly_contract_minutes: 0,
                        deleted_at: None,
                        created_at: now,
                        updated_at: now,
                    })
                    .await?;

                let employee_id = created.id;
                created_employees.push(created);
                Ok(employee_id)
            }
        }
    }

    pub async fn soft_delete_task(&mut self, id: TaskId) -> Result<(), CoreError> {
        self.get_task(id).await?;
        self.task_repository.soft_delete(id, Utc::now()).await
    }
}

fn validate_title(title: &str) -> Result<(), CoreError> {
    if title.trim().is_empty() {
        return Err(CoreError::Conflict("task title cannot be blank".to_owned()));
    }

    Ok(())
}

/// Mirrors `chk_tasks_dates_both_or_neither`/`chk_tasks_root_has_dates`/
/// `chk_tasks_ends_at_after_starts_at` in the domain, ahead of the trip to
/// the database.
fn validate_task_dates(
    has_parent: bool,
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
) -> Result<(), CoreError> {
    if starts_at.is_none() != ends_at.is_none() {
        return Err(CoreError::Conflict(
            "a task's starts_at and ends_at must be both set or both absent".to_owned(),
        ));
    }

    if !has_parent && starts_at.is_none() {
        return Err(CoreError::Conflict(
            "a root task must carry its own starts_at/ends_at".to_owned(),
        ));
    }

    if let (Some(starts_at), Some(ends_at)) = (starts_at, ends_at)
        && ends_at <= starts_at
    {
        return Err(CoreError::Conflict(
            "task ends_at must be after starts_at".to_owned(),
        ));
    }

    Ok(())
}

/// Mirrors `chk_tasks_customer_both_or_neither`: a task with a customer
/// carries both `customer_id` and `customer_context_id`, or neither.
fn validate_customer_pairing(
    customer_id: Option<CustomerId>,
    customer_context_id: Option<CustomerContextId>,
) -> Result<(), CoreError> {
    if customer_id.is_some() != customer_context_id.is_some() {
        return Err(CoreError::Conflict(
            "a task's customer_id and customer_context_id must be both set or both absent"
                .to_owned(),
        ));
    }

    Ok(())
}

fn validate_text_field(label: &str, value: &Option<String>) -> Result<(), CoreError> {
    if value.as_deref().is_some_and(|v| v.trim().is_empty()) {
        return Err(CoreError::Conflict(format!("{label} cannot be blank")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssigneeRef, CustomerContextId, CustomerId, OrganizationId, TaskStatus, User, UserId,
        domain::{
            employee::ports::MockEmployeeRepository, member::ports::MockMemberRepository,
            task::ports::MockTaskRepository, user::ports::MockUserRepository,
        },
    };
    use mockall::predicate::eq;
    use uuid::Uuid;

    /// A root task with sensible defaults ("Toiture", a customer, planned,
    /// blocking) — tests override only the fields they care about via struct
    /// update syntax.
    fn task(id: TaskId, organization_id: OrganizationId) -> Task {
        let now = Utc::now();
        Task {
            id,
            organization_id,
            parent_task_id: None,
            title: "Toiture".to_owned(),
            description: None,
            starts_at: Some(now),
            ends_at: Some(now + chrono::Duration::hours(2)),
            all_day: false,
            status: TaskStatus::Planned,
            blocks_availability: true,
            customer_id: Some(CustomerId(Uuid::new_v4())),
            customer_context_id: Some(CustomerContextId(Uuid::new_v4())),
            quote_id: None,
            assignments: Vec::new(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn employee(id: EmployeeId, organization_id: OrganizationId) -> Employee {
        let now = Utc::now();
        Employee {
            id,
            organization_id,
            user_id: None,
            last_name: "Alice".to_owned(),
            first_name: None,
            hourly_rate_cents: Some(3500),
            weekly_contract_minutes: 2100,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    // -- resolve_task_window ------------------------------------------------

    mod resolve_task_window_tests {
        use super::*;

        #[test]
        fn a_root_without_a_parent_uses_its_own_window() {
            let root = task(TaskId(Uuid::new_v4()), OrganizationId(Uuid::new_v4()));

            let resolved = resolve_task_window(&root, None);

            assert_eq!(resolved.starts_at, root.starts_at.unwrap());
            assert_eq!(resolved.ends_at, root.ends_at.unwrap());
        }

        #[test]
        fn a_subtask_without_dates_inherits_the_parent_window_exactly() {
            let organization_id = OrganizationId(Uuid::new_v4());
            let parent = task(TaskId(Uuid::new_v4()), organization_id);
            let subtask = Task {
                parent_task_id: Some(parent.id),
                starts_at: None,
                ends_at: None,
                ..task(TaskId(Uuid::new_v4()), organization_id)
            };

            let resolved = resolve_task_window(&subtask, Some(&parent));

            assert_eq!(resolved.starts_at, parent.starts_at.unwrap());
            assert_eq!(resolved.ends_at, parent.ends_at.unwrap());
        }

        #[test]
        fn a_subtask_with_its_own_dates_keeps_them_instead_of_the_parents() {
            let organization_id = OrganizationId(Uuid::new_v4());
            let parent = task(TaskId(Uuid::new_v4()), organization_id);

            let own_starts_at = parent.starts_at.unwrap() + chrono::Duration::hours(1);
            let own_ends_at = own_starts_at + chrono::Duration::minutes(30);
            let subtask = Task {
                parent_task_id: Some(parent.id),
                starts_at: Some(own_starts_at),
                ends_at: Some(own_ends_at),
                ..task(TaskId(Uuid::new_v4()), organization_id)
            };

            let resolved = resolve_task_window(&subtask, Some(&parent));

            assert_eq!(resolved.starts_at, own_starts_at);
            assert_eq!(resolved.ends_at, own_ends_at);
            assert_ne!(resolved.starts_at, parent.starts_at.unwrap());
        }
    }

    // -- validate_parent_depth -----------------------------------------------

    mod validate_parent_depth_tests {
        use super::*;

        #[test]
        fn no_candidate_parent_is_always_accepted() {
            assert!(validate_parent_depth(None).is_ok());
        }

        #[test]
        fn a_candidate_parent_that_is_itself_a_root_is_accepted() {
            let root = task(TaskId(Uuid::new_v4()), OrganizationId(Uuid::new_v4()));

            assert!(validate_parent_depth(Some(&root)).is_ok());
        }

        #[test]
        fn a_candidate_parent_that_already_has_a_parent_is_rejected() {
            let organization_id = OrganizationId(Uuid::new_v4());
            let grandparent_id = TaskId(Uuid::new_v4());
            let candidate_parent = Task {
                parent_task_id: Some(grandparent_id),
                ..task(TaskId(Uuid::new_v4()), organization_id)
            };

            let err = validate_parent_depth(Some(&candidate_parent)).unwrap_err();

            assert!(matches!(err, CoreError::Conflict(_)));
        }
    }

    // -- TaskService -----------------------------------------------------------

    #[allow(clippy::type_complexity)]
    fn service(
        task_repository: MockTaskRepository,
        employee_repository: MockEmployeeRepository,
        user_repository: MockUserRepository,
        member_repository: MockMemberRepository,
    ) -> TaskService<
        MockTaskRepository,
        MockEmployeeRepository,
        MockUserRepository,
        MockMemberRepository,
    > {
        TaskService::new(
            task_repository,
            employee_repository,
            user_repository,
            member_repository,
        )
    }

    fn create_command() -> CreateTaskCommand {
        let now = Utc::now();
        CreateTaskCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            parent_task_id: None,
            title: "Toiture".to_owned(),
            description: None,
            starts_at: Some(now),
            ends_at: Some(now + chrono::Duration::hours(2)),
            all_day: false,
            blocks_availability: true,
            customer_id: Some(CustomerId(Uuid::new_v4())),
            customer_context_id: Some(CustomerContextId(Uuid::new_v4())),
            quote_id: None,
        }
    }

    #[tokio::test]
    async fn create_task_persists_with_planned_status_and_no_assignments() {
        let mut task_repository = MockTaskRepository::new();
        task_repository.expect_insert().times(1).returning(|t| {
            let cloned = t.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let created = service.create_task(create_command()).await.unwrap();

        assert_eq!(created.status, TaskStatus::Planned);
        assert!(created.assignments.is_empty());
        assert_eq!(created.title, "Toiture");
    }

    #[tokio::test]
    async fn create_task_persists_a_task_without_a_customer() {
        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_insert()
            .withf(|t| t.customer_id.is_none() && t.customer_context_id.is_none())
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.customer_id = None;
        command.customer_context_id = None;

        let created = service.create_task(command).await.unwrap();

        assert!(created.customer_id.is_none());
        assert!(created.customer_context_id.is_none());
    }

    #[tokio::test]
    async fn create_task_rejects_ends_at_before_starts_at() {
        let mut service = service(
            MockTaskRepository::new(),
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.ends_at = command.starts_at;

        let err = service.create_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_rejects_blank_title() {
        let mut service = service(
            MockTaskRepository::new(),
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.title = "   ".to_owned();

        let err = service.create_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_rejects_a_customer_without_a_customer_context() {
        let mut service = service(
            MockTaskRepository::new(),
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.customer_context_id = None;

        let err = service.create_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_rejects_a_root_without_dates() {
        let mut service = service(
            MockTaskRepository::new(),
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.starts_at = None;
        command.ends_at = None;

        let err = service.create_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_allows_a_subtask_without_dates_and_without_assignees() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let parent_id = TaskId(Uuid::new_v4());
        let parent = task(parent_id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(parent_id))
            .returning(move |_| {
                let parent = parent.clone();
                Box::pin(async move { Ok(Some(parent)) })
            });
        task_repository.expect_insert().returning(|t| {
            let cloned = t.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.organization_id = organization_id;
        command.parent_task_id = Some(parent_id);
        command.starts_at = None;
        command.ends_at = None;

        let created = service.create_task(command).await.unwrap();

        assert_eq!(created.parent_task_id, Some(parent_id));
        assert!(created.starts_at.is_none());
        assert!(created.ends_at.is_none());
        assert!(
            created.assignments.is_empty(),
            "a subtask never inherits its parent's assignees"
        );
    }

    #[tokio::test]
    async fn create_task_rejects_a_subtask_whose_parent_already_has_a_parent() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let parent_id = TaskId(Uuid::new_v4());
        let grandparent_already_a_parent = Task {
            parent_task_id: Some(TaskId(Uuid::new_v4())),
            ..task(parent_id, organization_id)
        };

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(parent_id))
            .returning(move |_| {
                let parent = grandparent_already_a_parent.clone();
                Box::pin(async move { Ok(Some(parent)) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.organization_id = organization_id;
        command.parent_task_id = Some(parent_id);
        command.starts_at = None;
        command.ends_at = None;

        let err = service.create_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_rejects_a_parent_from_another_organization() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let other_org_id = OrganizationId(Uuid::new_v4());
        let parent_id = TaskId(Uuid::new_v4());
        let foreign_parent = task(parent_id, other_org_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(parent_id))
            .returning(move |_| {
                let parent = foreign_parent.clone();
                Box::pin(async move { Ok(Some(parent)) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.organization_id = organization_id;
        command.parent_task_id = Some(parent_id);
        command.starts_at = None;
        command.ends_at = None;

        let err = service.create_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn get_task_returns_not_found_when_missing() {
        let id = TaskId(Uuid::new_v4());
        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let err = service.get_task(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_tasks_delegates_to_repo_and_attaches_child_counts() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let root_id = TaskId(Uuid::new_v4());
        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_list_by_organization()
            .withf(move |org, parent, limit, offset| {
                *org == organization_id && parent.is_none() && *limit == 10 && *offset == 20
            })
            .returning(move |org_id, _, _, _| {
                Box::pin(async move { Ok((vec![task(root_id, org_id)], 1)) })
            });
        task_repository
            .expect_count_children()
            .withf(move |ids| ids == [root_id])
            .returning(move |_| Box::pin(async move { Ok(HashMap::from([(root_id, 2)])) }));

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let (items, child_counts, total) = service
            .list_tasks(organization_id, None, 10, 20)
            .await
            .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(child_counts.get(&root_id), Some(&2));
    }

    #[tokio::test]
    async fn patch_task_reschedules_without_touching_assignees() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let new_starts_at = existing.starts_at.unwrap() + chrono::Duration::days(1);
        let new_ends_at = existing.ends_at.unwrap() + chrono::Duration::days(1);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(move |t| t.assignments.is_empty())
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.starts_at = Some(Some(new_starts_at));
        command.ends_at = Some(Some(new_ends_at));

        let (updated, created_employees) = service.patch_task(command).await.unwrap();

        assert_eq!(updated.starts_at, Some(new_starts_at));
        assert_eq!(updated.ends_at, Some(new_ends_at));
        assert!(created_employees.is_empty());
    }

    #[tokio::test]
    async fn patch_task_rejects_ends_at_before_merged_starts_at() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let existing_starts_at = existing.starts_at.unwrap();

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.ends_at = Some(Some(existing_starts_at - chrono::Duration::hours(1)));

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_task_updates_the_title() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(|t| t.title == "Nouveau titre")
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.title = Some("Nouveau titre".to_owned());

        let (updated, _) = service.patch_task(command).await.unwrap();

        assert_eq!(updated.title, "Nouveau titre");
    }

    #[tokio::test]
    async fn patch_task_rejects_a_blank_title() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.title = Some("   ".to_owned());

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_task_can_clear_the_description() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = Task {
            description: Some("initial".to_owned()),
            ..task(id, organization_id)
        };

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(|t| t.description.is_none())
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.description = Some(None);

        let (updated, _) = service.patch_task(command).await.unwrap();

        assert!(updated.description.is_none());
    }

    #[tokio::test]
    async fn patch_task_can_clear_its_own_dates_to_inherit_from_a_parent() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let parent_id = TaskId(Uuid::new_v4());
        let existing = Task {
            parent_task_id: Some(parent_id),
            ..task(id, organization_id)
        };

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(|t| t.starts_at.is_none() && t.ends_at.is_none())
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.starts_at = Some(None);
        command.ends_at = Some(None);

        let (updated, _) = service.patch_task(command).await.unwrap();

        assert!(updated.starts_at.is_none());
        assert!(updated.ends_at.is_none());
    }

    #[tokio::test]
    async fn patch_task_rejects_clearing_dates_on_a_root() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.starts_at = Some(None);
        command.ends_at = Some(None);

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_task_rejects_designating_itself_as_parent() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.parent_task_id = Some(Some(id));

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_task_rejects_reparenting_under_a_task_that_already_has_a_parent() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let candidate_parent_id = TaskId(Uuid::new_v4());
        let candidate_parent = Task {
            parent_task_id: Some(TaskId(Uuid::new_v4())),
            ..task(candidate_parent_id, organization_id)
        };

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_find_by_id()
            .with(eq(candidate_parent_id))
            .returning(move |_| {
                let parent = candidate_parent.clone();
                Box::pin(async move { Ok(Some(parent)) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.parent_task_id = Some(Some(candidate_parent_id));

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_task_can_toggle_blocks_availability() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        assert!(existing.blocks_availability);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(|t| !t.blocks_availability)
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.blocks_availability = Some(false);

        let (updated, _) = service.patch_task(command).await.unwrap();

        assert!(!updated.blocks_availability);
    }

    #[tokio::test]
    async fn patch_task_assigns_an_existing_employee_in_the_same_org() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let employee_id = EmployeeId(Uuid::new_v4());
        let target_employee = employee(employee_id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(move |t| t.assignments.len() == 1 && t.assignments[0].employee_id == employee_id)
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_id()
            .with(eq(employee_id))
            .returning(move |_| {
                let e = target_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut service = service(
            task_repository,
            employee_repository,
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Employee(employee_id)]);

        let (updated, created_employees) = service.patch_task(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
        assert_eq!(updated.assignments[0].employee_id, employee_id);
        assert!(created_employees.is_empty());
    }

    #[tokio::test]
    async fn patch_task_rejects_an_employee_from_another_organization() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let other_org_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let employee_id = EmployeeId(Uuid::new_v4());
        let foreign_employee = employee(employee_id, other_org_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_id()
            .with(eq(employee_id))
            .returning(move |_| {
                let e = foreign_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut service = service(
            task_repository,
            employee_repository,
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Employee(employee_id)]);

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn patch_task_creates_an_employee_for_a_member_without_one() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let user_id = UserId(Uuid::new_v4());

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(|t| t.assignments.len() == 1)
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .with(eq(organization_id), eq(user_id))
            .returning(move |org_id, user_id| {
                let m = crate::Member {
                    id: crate::MemberId(Uuid::new_v4()),
                    organization_id: org_id,
                    user_id,
                    joined_at: Utc::now(),
                };
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_user_id()
            .with(eq(organization_id), eq(user_id))
            .returning(|_, _| Box::pin(async { Ok(None) }));
        employee_repository
            .expect_insert()
            .withf(move |e| {
                e.user_id == Some(user_id)
                    && e.hourly_rate_cents.is_none()
                    && e.weekly_contract_minutes == 0
                    && e.last_name == "Bob Member"
                    && e.first_name.is_none()
            })
            .times(1)
            .returning(|e| {
                let cloned = e.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_find_by_id()
            .with(eq(user_id))
            .returning(move |id| {
                let now = Utc::now();
                let user = User {
                    id,
                    email: "bob@example.com".to_owned(),
                    username: "bob".to_owned(),
                    name: "Bob Member".to_owned(),
                    sub: "sub-bob".to_owned(),
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(user)) })
            });

        let mut service = service(
            task_repository,
            employee_repository,
            user_repository,
            member_repository,
        );

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Member(user_id)]);

        let (updated, created_employees) = service.patch_task(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
        assert_eq!(created_employees.len(), 1);
        assert_eq!(created_employees[0].hourly_rate_cents, None);
        assert_eq!(created_employees[0].weekly_contract_minutes, 0);
    }

    #[tokio::test]
    async fn patch_task_reuses_an_employee_already_provisioned_for_the_member() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let user_id = UserId(Uuid::new_v4());
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut existing_employee = employee(employee_id, organization_id);
        existing_employee.user_id = Some(user_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(move |t| t.assignments.len() == 1 && t.assignments[0].employee_id == employee_id)
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .with(eq(organization_id), eq(user_id))
            .returning(move |org_id, user_id| {
                let m = crate::Member {
                    id: crate::MemberId(Uuid::new_v4()),
                    organization_id: org_id,
                    user_id,
                    joined_at: Utc::now(),
                };
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_user_id()
            .with(eq(organization_id), eq(user_id))
            .returning(move |_, _| {
                let e = existing_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });
        // No `expect_insert`: reusing the existing record must not create a
        // second one for the same person.

        let mut service = service(
            task_repository,
            employee_repository,
            MockUserRepository::new(),
            member_repository,
        );

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Member(user_id)]);

        let (updated, created_employees) = service.patch_task(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
        assert_eq!(updated.assignments[0].employee_id, employee_id);
        assert!(created_employees.is_empty());
    }

    #[tokio::test]
    async fn patch_task_rejects_a_member_outside_the_organization() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let user_id = UserId(Uuid::new_v4());

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .with(eq(organization_id), eq(user_id))
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            member_repository,
        );

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Member(user_id)]);

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn patch_task_dedupes_repeated_assignees() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let employee_id = EmployeeId(Uuid::new_v4());
        let target_employee = employee(employee_id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(|t| t.assignments.len() == 1)
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_id()
            .with(eq(employee_id))
            .returning(move |_| {
                let e = target_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut service = service(
            task_repository,
            employee_repository,
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![
            AssigneeRef::Employee(employee_id),
            AssigneeRef::Employee(employee_id),
        ]);

        let (updated, _) = service.patch_task(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
    }

    #[tokio::test]
    async fn patch_task_can_clear_all_assignees() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut existing = task(id, organization_id);
        existing.assignments.push(TaskAssignment {
            id: TaskAssignmentId(Uuid::new_v4()),
            organization_id,
            task_id: id,
            employee_id: EmployeeId(Uuid::new_v4()),
            created_at: Utc::now(),
        });

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_update()
            .withf(|t| t.assignments.is_empty())
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(Vec::new());

        let (updated, _) = service.patch_task(command).await.unwrap();

        assert!(updated.assignments.is_empty());
    }

    #[tokio::test]
    async fn soft_delete_task_checks_existence_then_deletes() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = service(
            task_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        service.soft_delete_task(id).await.unwrap();
    }
}
