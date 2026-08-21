use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};

use crate::{
    CustomerContextId, CustomerId, MemberId, OrganizationId, Task, TimeRange,
    domain::{
        member::ports::MemberRepository,
        task::{
            TaskAssignment, TaskAssignmentId, TaskId,
            commands::{CreateTaskCommand, PatchTaskCommand},
            ports::TaskRepository,
        },
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

/// Refuses to turn a task into a subtask when it already has children of its
/// own. `validate_parent_depth` asks whether a *candidate parent* is
/// acceptable; this asks the reciprocal question, about the task being
/// moved — reparenting a task with children would silently push those
/// children to a third level, which is exactly the invariant
/// `validate_parent_depth` exists to prevent from the other direction.
///
/// `child_count` is supplied by the caller (`TaskRepository::count_children`,
/// already loaded at the application layer for `GET /tasks`) rather than
/// fetched here, so this stays pure and I/O-free like its sibling. A task
/// that stays a root, whatever else a `PATCH` changes about it (dates,
/// title, assignees), never needs this check — only an attempt to set
/// `parent_task_id` does.
pub fn validate_reparenting(child_count: i64) -> Result<(), CoreError> {
    if child_count > 0 {
        return Err(CoreError::Conflict(
            "a task with children cannot become a subtask".to_owned(),
        ));
    }

    Ok(())
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
pub struct TaskService<TR, MR>
where
    TR: TaskRepository,
    MR: MemberRepository,
{
    task_repository: TR,
    member_repository: MR,
}

impl<TR, MR> TaskService<TR, MR>
where
    TR: TaskRepository,
    MR: MemberRepository,
{
    /// Neither an `EmployeeRepository` nor a `UserRepository`: assigning a
    /// task needs the member to exist in the organization, and nothing else.
    /// The first was used to provision an HR record on the fly, the second to
    /// name it — a seat carries its own name, so both are gone.
    pub fn new(task_repository: TR, member_repository: MR) -> Self {
        Self {
            task_repository,
            member_repository,
        }
    }

    /// One member's jobs overlapping a day, for the field app.
    ///
    /// A read, so no authorization here: the caller has already resolved the
    /// member from the connected account, which is the check that matters.
    pub async fn list_assigned_to_member_on(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        day_starts_at: DateTime<Utc>,
        day_ends_at: DateTime<Utc>,
    ) -> Result<Vec<Task>, CoreError> {
        self.task_repository
            .list_for_assignee_on(organization_id, member_id, day_starts_at, day_ends_at)
            .await
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
                project_id: command.project_id,
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
    pub async fn patch_task(&mut self, command: PatchTaskCommand) -> Result<Task, CoreError> {
        let mut task = self.get_task(command.id).await?;

        if let Some(parent_choice) = command.parent_task_id {
            match parent_choice {
                Some(parent_id) => {
                    if parent_id == task.id {
                        return Err(CoreError::Conflict(
                            "a task cannot be its own parent".to_owned(),
                        ));
                    }

                    // Checked before the parent lookup: a task with
                    // children of its own can never become a subtask,
                    // whatever the candidate parent looks like — see
                    // `validate_reparenting`.
                    let child_counts = self.task_repository.count_children(&[task.id]).await?;
                    let child_count = child_counts.get(&task.id).copied().unwrap_or(0);
                    validate_reparenting(child_count)?;

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
        if let Some(project_id) = command.project_id {
            task.project_id = project_id;
        }
        task.updated_at = Utc::now();

        if let Some(assignees) = command.assignees {
            task.assignments = self.resolve_assignments(&task, assignees).await?;
        }

        self.task_repository.update(&task).await
    }

    /// Resolves each `AssigneeRef` to a concrete, deduplicated `employee_id`
    /// list, provisioning an employee record on the fly for `member`
    /// assignees who don't have one yet (`hourly_rate_cents` stays `NULL`
    /// — never defaulted to `0`, which would read as "genuinely free").
    async fn resolve_assignments(
        &mut self,
        task: &Task,
        assignees: Vec<crate::AssigneeRef>,
    ) -> Result<Vec<TaskAssignment>, CoreError> {
        let mut seen = HashSet::new();
        let mut member_ids = Vec::with_capacity(assignees.len());

        for assignee in assignees {
            let member_id = self.resolve_assignee(task, assignee).await?;

            if seen.insert(member_id) {
                member_ids.push(member_id);
            }
        }

        let now = Utc::now();
        let assignments = member_ids
            .into_iter()
            .map(|member_id| TaskAssignment {
                id: TaskAssignmentId(generate_uuid_v7()),
                organization_id: task.organization_id,
                task_id: task.id,
                member_id,
                created_at: now,
            })
            .collect();

        Ok(assignments)
    }

    /// Checks that the assignee is a member of the task's organization, and
    /// returns it.
    ///
    /// This used to branch on the reference's kind and, for a bare member,
    /// provision an employee record on the fly — named from the account's
    /// `display_name`, with no rate. All of that existed because only an
    /// employee could be assigned. A member is assignable as it stands, so the
    /// provisioning is gone and this is now a membership check, nothing more.
    async fn resolve_assignee(
        &mut self,
        task: &Task,
        assignee: crate::AssigneeRef,
    ) -> Result<MemberId, CoreError> {
        let crate::AssigneeRef(member_id) = assignee;

        let member = self
            .member_repository
            .find_by_id(member_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        // Checked against the task's organization, not a caller-supplied one:
        // assigning across tenants must read as "no such member", never as a
        // successful assignment.
        if member.organization_id != task.organization_id {
            return Err(CoreError::NotFound);
        }

        Ok(member_id)
    }

    /// Assigns `assignees` to every task in `task_ids`, each replacing that
    /// task's complete assignment set — same contract as `patch_task`'s own
    /// `assignees`, applied to many tasks in one call instead of one HTTP
    /// round trip per task. Every task must belong to `organization_id`; the
    /// first missing task, or one from another organization, fails the
    /// whole call before its own write and before any later task is even
    /// looked at. Since the caller wraps this in one transaction
    /// (`#[transactional(task, member)]` on
    /// `MestierUseCase::bulk_assign_tasks`), a failure partway through rolls
    /// back every earlier task's write too — never a partial batch.
    pub async fn bulk_assign_tasks(
        &mut self,
        organization_id: OrganizationId,
        task_ids: Vec<TaskId>,
        assignees: Vec<crate::AssigneeRef>,
    ) -> Result<Vec<Task>, CoreError> {
        let mut updated = Vec::with_capacity(task_ids.len());

        for task_id in task_ids {
            let mut task = self.get_task(task_id).await?;
            if task.organization_id != organization_id {
                return Err(CoreError::NotFound);
            }

            task.assignments = self.resolve_assignments(&task, assignees.clone()).await?;
            task.updated_at = Utc::now();
            updated.push(self.task_repository.update(&task).await?);
        }

        Ok(updated)
    }

    /// Soft-deletes `id` together with every direct child it has. The
    /// two-level nesting cap (`validate_parent_depth`) means a child never
    /// has children of its own, so cascading one level down is the whole
    /// tree — no recursion needed.
    ///
    /// `ON DELETE CASCADE` on `tasks.parent_task_id` only fires for a
    /// physical delete; nothing in the schema cascades a *soft* delete, so
    /// without this a deleted root would leave its subtasks behind,
    /// pointing at a parent that no longer exists — and, for the ones with
    /// no dates of their own, with no window left to resolve at all (see
    /// `resolve_task_window`). Every write here lands in the same
    /// transaction as the caller (`#[transactional(task, ...)]` on
    /// `MestierUseCase::soft_delete_task`), so a failure partway through
    /// rolls back the whole cascade rather than leaving it half-applied.
    pub async fn soft_delete_task(&mut self, id: TaskId) -> Result<(), CoreError> {
        let task = self.get_task(id).await?;
        let now = Utc::now();

        // `i64::MAX` rather than `u64::MAX`: the Postgres adapter casts
        // `limit` to `i64` for the SQL `LIMIT` clause, and `u64::MAX as i64`
        // wraps around to `-1`, which Postgres rejects outright. A parent's
        // own direct children are never numerous enough for a real cap to
        // matter — this is "no limit", not a page size.
        let (children, _total) = self
            .task_repository
            .list_by_organization(task.organization_id, Some(id), i64::MAX as u64, 0)
            .await?;

        for child in &children {
            self.task_repository.soft_delete(child.id, now).await?;
        }

        self.task_repository.soft_delete(id, now).await
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

/// Mirrors `chk_tasks_context_requires_customer`: a `customer_context_id`
/// always implies its `customer_id` — a customer_context belongs to exactly
/// one customer, so naming one without the other makes no sense. The
/// reverse is not required: a task may carry a `customer_id` with no
/// `customer_context_id` at all — "linked to this client, no particular
/// site yet" — distinct from no client (an internal task/meeting).
fn validate_customer_pairing(
    customer_id: Option<CustomerId>,
    customer_context_id: Option<CustomerContextId>,
) -> Result<(), CoreError> {
    if customer_context_id.is_some() && customer_id.is_none() {
        return Err(CoreError::Conflict(
            "a task's customer_context_id requires a customer_id".to_owned(),
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
        AssigneeRef, CustomerContextId, CustomerId, Member, MemberId, OrganizationId, TaskStatus,
        domain::{member::ports::MockMemberRepository, task::ports::MockTaskRepository},
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
            project_id: None,
            assignments: Vec::new(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// An assignment points at a seat, so this is what the tests build now —
    /// the contractual profile plays no part in resolving an assignee.
    fn member(id: MemberId, organization_id: OrganizationId) -> Member {
        Member {
            id,
            organization_id,
            user_id: None,
            last_name: "Alice".to_owned(),
            first_name: None,
            joined_at: None,
            created_at: Utc::now(),
            deleted_at: None,
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

    // -- validate_reparenting -------------------------------------------------

    mod validate_reparenting_tests {
        use super::*;

        #[test]
        fn a_task_with_no_children_may_be_reparented() {
            assert!(validate_reparenting(0).is_ok());
        }

        #[test]
        fn a_task_with_children_is_rejected_as_a_subtask() {
            let err = validate_reparenting(2).unwrap_err();

            assert!(matches!(err, CoreError::Conflict(_)));
        }
    }

    // -- TaskService -----------------------------------------------------------

    #[allow(clippy::type_complexity)]
    fn service(
        task_repository: MockTaskRepository,
        member_repository: MockMemberRepository,
    ) -> TaskService<MockTaskRepository, MockMemberRepository> {
        TaskService::new(task_repository, member_repository)
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
            project_id: None,
        }
    }

    #[tokio::test]
    async fn create_task_persists_with_planned_status_and_no_assignments() {
        let mut task_repository = MockTaskRepository::new();
        task_repository.expect_insert().times(1).returning(|t| {
            let cloned = t.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = service(task_repository, MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = create_command();
        command.customer_id = None;
        command.customer_context_id = None;

        let created = service.create_task(command).await.unwrap();

        assert!(created.customer_id.is_none());
        assert!(created.customer_context_id.is_none());
    }

    #[tokio::test]
    async fn create_task_rejects_ends_at_before_starts_at() {
        let mut service = service(MockTaskRepository::new(), MockMemberRepository::new());

        let mut command = create_command();
        command.ends_at = command.starts_at;

        let err = service.create_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_rejects_blank_title() {
        let mut service = service(MockTaskRepository::new(), MockMemberRepository::new());

        let mut command = create_command();
        command.title = "   ".to_owned();

        let err = service.create_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_allows_a_customer_without_a_customer_context() {
        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_insert()
            .withf(|t| t.customer_id.is_some() && t.customer_context_id.is_none())
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = create_command();
        command.customer_context_id = None;

        let created = service.create_task(command).await.unwrap();

        assert!(created.customer_id.is_some());
        assert!(created.customer_context_id.is_none());
    }

    #[tokio::test]
    async fn create_task_rejects_a_customer_context_without_a_customer() {
        let mut service = service(MockTaskRepository::new(), MockMemberRepository::new());

        let mut command = create_command();
        command.customer_id = None;
        // `customer_context_id` stays `Some(..)` from `create_command`'s default —
        // a context always implies its customer, even though a customer no
        // longer implies a context (see the sibling test just above).

        let err = service.create_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_rejects_a_root_without_dates() {
        let mut service = service(MockTaskRepository::new(), MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.starts_at = Some(Some(new_starts_at));
        command.ends_at = Some(Some(new_ends_at));

        let updated = service.patch_task(command).await.unwrap();

        assert_eq!(updated.starts_at, Some(new_starts_at));
        assert_eq!(updated.ends_at, Some(new_ends_at));
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

        let mut service = service(task_repository, MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.title = Some("Nouveau titre".to_owned());

        let updated = service.patch_task(command).await.unwrap();

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

        let mut service = service(task_repository, MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.description = Some(None);

        let updated = service.patch_task(command).await.unwrap();

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

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.starts_at = Some(None);
        command.ends_at = Some(None);

        let updated = service.patch_task(command).await.unwrap();

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

        let mut service = service(task_repository, MockMemberRepository::new());

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

        let mut service = service(task_repository, MockMemberRepository::new());

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
            .expect_count_children()
            .withf(move |ids| ids == [id])
            .returning(|_| Box::pin(async { Ok(HashMap::new()) }));
        task_repository
            .expect_find_by_id()
            .with(eq(candidate_parent_id))
            .returning(move |_| {
                let parent = candidate_parent.clone();
                Box::pin(async move { Ok(Some(parent)) })
            });

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.parent_task_id = Some(Some(candidate_parent_id));

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_task_reparents_a_childless_task_under_a_root() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let new_parent_id = TaskId(Uuid::new_v4());
        let new_parent = task(new_parent_id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_count_children()
            .withf(move |ids| ids == [id])
            .returning(|_| Box::pin(async { Ok(HashMap::new()) }));
        task_repository
            .expect_find_by_id()
            .with(eq(new_parent_id))
            .returning(move |_| {
                let parent = new_parent.clone();
                Box::pin(async move { Ok(Some(parent)) })
            });
        task_repository
            .expect_update()
            .withf(move |t| t.parent_task_id == Some(new_parent_id))
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.parent_task_id = Some(Some(new_parent_id));

        let updated = service.patch_task(command).await.unwrap();

        assert_eq!(updated.parent_task_id, Some(new_parent_id));
    }

    #[tokio::test]
    async fn patch_task_rejects_reparenting_a_task_that_has_children() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let new_parent_id = TaskId(Uuid::new_v4());

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        task_repository
            .expect_count_children()
            .withf(move |ids| ids == [id])
            .returning(move |_| Box::pin(async move { Ok(HashMap::from([(id, 2)])) }));
        // No `expect_find_by_id` for `new_parent_id`, no `expect_update`: the
        // rejection must land before either — a task with children never
        // even gets to the point of checking the candidate parent.

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.parent_task_id = Some(Some(new_parent_id));

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_task_keeps_a_task_with_children_editable_while_it_stays_a_root() {
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
        // No `expect_count_children`: leaving `parent_task_id` untouched must
        // never trigger the reparenting check — a root with children stays
        // freely editable (mockall panics if an unexpected method is called,
        // which is exactly the assertion this test relies on).
        task_repository
            .expect_update()
            .withf(|t| {
                t.title == "Nouveau titre" && t.assignments.len() == 1 && t.parent_task_id.is_none()
            })
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let member_id = MemberId(Uuid::new_v4());
        let target_member = member(member_id, organization_id);
        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(member_id))
            .returning(move |_| {
                let m = target_member.clone();
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut service = service(task_repository, member_repository);

        let mut command = PatchTaskCommand::new(id);
        command.title = Some("Nouveau titre".to_owned());
        command.assignees = Some(vec![AssigneeRef(member_id)]);

        let updated = service.patch_task(command).await.unwrap();

        assert_eq!(updated.title, "Nouveau titre");
        assert_eq!(updated.assignments.len(), 1);
    }

    #[tokio::test]
    async fn patch_task_rejects_clearing_the_parent_of_a_dateless_subtask() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let parent_id = TaskId(Uuid::new_v4());
        let existing = Task {
            parent_task_id: Some(parent_id),
            starts_at: None,
            ends_at: None,
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
        // No `expect_update`: turning this subtask into a dateless root must
        // be rejected before any write is attempted.

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.parent_task_id = Some(None);

        let err = service.patch_task(command).await.unwrap_err();

        assert!(
            matches!(err, CoreError::Conflict(_)),
            "a root without its own dates must be rejected, even when the \
             dates were already absent before the PATCH — clearing the \
             parent is what turns the missing dates into a violation"
        );
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

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.blocks_availability = Some(false);

        let updated = service.patch_task(command).await.unwrap();

        assert!(!updated.blocks_availability);
    }

    #[tokio::test]
    async fn patch_task_assigns_an_existing_employee_in_the_same_org() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let member_id = MemberId(Uuid::new_v4());
        let target_member = member(member_id, organization_id);

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
            .withf(move |t| t.assignments.len() == 1 && t.assignments[0].member_id == member_id)
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(member_id))
            .returning(move |_| {
                let m = target_member.clone();
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut service = service(task_repository, member_repository);

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef(member_id)]);

        let updated = service.patch_task(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
        assert_eq!(updated.assignments[0].member_id, member_id);
    }

    #[tokio::test]
    async fn patch_task_rejects_an_employee_from_another_organization() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let other_org_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let member_id = MemberId(Uuid::new_v4());
        let foreign_member = member(member_id, other_org_id);

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
            .expect_find_by_id()
            .with(eq(member_id))
            .returning(move |_| {
                let m = foreign_member.clone();
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut service = service(task_repository, member_repository);

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef(member_id)]);

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    /// A member with no contractual profile is assignable as it stands.
    ///
    /// Two tests used to live here: one asserting that assigning a bare member
    /// created an employee record on the fly, another that a second assignment
    /// reused it. `TaskService` no longer holds an `EmployeeRepository` at all,
    /// so what is left to check is that resolving an assignee touches the seat
    /// and nothing else.
    #[tokio::test]
    async fn patch_task_assigns_a_member_that_has_no_profile() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let member_id = MemberId(Uuid::new_v4());
        let seat = member(member_id, organization_id);

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
            .withf(move |t| t.assignments.len() == 1 && t.assignments[0].member_id == member_id)
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(member_id))
            .returning(move |_| {
                let seat = seat.clone();
                Box::pin(async move { Ok(Some(seat)) })
            });

        let mut service = service(task_repository, member_repository);

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef(member_id)]);

        let updated = service.patch_task(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
        assert_eq!(updated.assignments[0].member_id, member_id);
    }

    #[tokio::test]
    async fn patch_task_rejects_a_member_outside_the_organization() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let member_id = MemberId(Uuid::new_v4());

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
            .expect_find_by_id()
            .with(eq(member_id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = service(task_repository, member_repository);

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef(member_id)]);

        let err = service.patch_task(command).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn patch_task_dedupes_repeated_assignees() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let member_id = MemberId(Uuid::new_v4());
        let target_member = member(member_id, organization_id);

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
            .expect_find_by_id()
            .with(eq(member_id))
            .returning(move |_| {
                let m = target_member.clone();
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut service = service(task_repository, member_repository);

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(vec![AssigneeRef(member_id), AssigneeRef(member_id)]);

        let updated = service.patch_task(command).await.unwrap();

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
            member_id: MemberId(Uuid::new_v4()),
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

        let mut service = service(task_repository, MockMemberRepository::new());

        let mut command = PatchTaskCommand::new(id);
        command.assignees = Some(Vec::new());

        let updated = service.patch_task(command).await.unwrap();

        assert!(updated.assignments.is_empty());
    }

    // -- bulk_assign_tasks ----------------------------------------------------

    #[tokio::test]
    async fn bulk_assign_tasks_assigns_the_same_set_to_every_task() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_a_id = TaskId(Uuid::new_v4());
        let task_b_id = TaskId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());
        let target_member = member(member_id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(task_a_id))
            .returning(move |_| {
                let t = task(task_a_id, organization_id);
                Box::pin(async move { Ok(Some(t)) })
            });
        task_repository
            .expect_find_by_id()
            .with(eq(task_b_id))
            .returning(move |_| {
                let t = task(task_b_id, organization_id);
                Box::pin(async move { Ok(Some(t)) })
            });
        task_repository
            .expect_update()
            .withf(move |t| t.assignments.len() == 1 && t.assignments[0].member_id == member_id)
            .times(2)
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(member_id))
            .returning(move |_| {
                let m = target_member.clone();
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut service = service(task_repository, member_repository);

        let updated = service
            .bulk_assign_tasks(
                organization_id,
                vec![task_a_id, task_b_id],
                vec![AssigneeRef(member_id)],
            )
            .await
            .unwrap();

        assert_eq!(updated.len(), 2);
        assert!(updated.iter().all(|t| t.assignments.len() == 1));
    }

    /// A mock only proves the loop short-circuits — it stops calling the
    /// repository the instant a task fails, never reaching later ids in the
    /// list. It cannot prove the *database* rolls back `task_a`'s own write,
    /// which already landed before the failure: that needs a real
    /// transaction, hence `bulk_assign_tasks_rolls_back_the_whole_batch_on_a_partial_failure`
    /// in `application::task::tests` — this test's counterpart against a
    /// live Postgres.
    #[tokio::test]
    async fn bulk_assign_tasks_stops_at_the_first_missing_task_never_reaching_the_rest() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_a_id = TaskId(Uuid::new_v4());
        let missing_id = TaskId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());
        let target_member = member(member_id, organization_id);

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(task_a_id))
            .returning(move |_| {
                let t = task(task_a_id, organization_id);
                Box::pin(async move { Ok(Some(t)) })
            });
        task_repository
            .expect_find_by_id()
            .with(eq(missing_id))
            .returning(|_| Box::pin(async { Ok(None) }));
        // `task_a` is processed (and written) before `missing_id` is even
        // looked at — the loop is sequential, so this call does happen at
        // the mock level. Only the DB-backed test can show it doesn't
        // survive the transaction's rollback.
        task_repository.expect_update().times(1).returning(|t| {
            let cloned = t.clone();
            Box::pin(async move { Ok(cloned) })
        });
        // No `expect_find_by_id` for a third id: never_reached_id never
        // even gets a lookup once `missing_id` fails — mockall panics on
        // an unexpected call, which is exactly the assertion this relies on.

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(member_id))
            .returning(move |_| {
                let m = target_member.clone();
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut service = service(task_repository, member_repository);

        let never_reached_id = TaskId(Uuid::new_v4());
        let err = service
            .bulk_assign_tasks(
                organization_id,
                vec![task_a_id, missing_id, never_reached_id],
                vec![AssigneeRef(member_id)],
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn bulk_assign_tasks_rejects_a_task_from_another_organization() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let other_org_id = OrganizationId(Uuid::new_v4());
        let foreign_task_id = TaskId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());

        let mut task_repository = MockTaskRepository::new();
        task_repository
            .expect_find_by_id()
            .with(eq(foreign_task_id))
            .returning(move |_| {
                let t = task(foreign_task_id, other_org_id);
                Box::pin(async move { Ok(Some(t)) })
            });
        // No `expect_update`: a task belonging to a different organization
        // must be rejected as `NotFound`, never written to.

        let mut service = service(task_repository, MockMemberRepository::new());

        let err = service
            .bulk_assign_tasks(
                organization_id,
                vec![foreign_task_id],
                vec![AssigneeRef(member_id)],
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
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
            .expect_list_by_organization()
            .withf(move |org, parent, _, _| *org == organization_id && *parent == Some(id))
            .returning(|_, _, _, _| Box::pin(async { Ok((Vec::new(), 0)) }));
        task_repository
            .expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = service(task_repository, MockMemberRepository::new());

        service.soft_delete_task(id).await.unwrap();
    }

    #[tokio::test]
    async fn soft_delete_task_cascades_to_every_direct_child() {
        let id = TaskId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = task(id, organization_id);
        let child_a_id = TaskId(Uuid::new_v4());
        let child_b_id = TaskId(Uuid::new_v4());
        let child_a = Task {
            parent_task_id: Some(id),
            starts_at: None,
            ends_at: None,
            ..task(child_a_id, organization_id)
        };
        let child_b = Task {
            parent_task_id: Some(id),
            ..task(child_b_id, organization_id)
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
            .expect_list_by_organization()
            .withf(move |org, parent, _, _| *org == organization_id && *parent == Some(id))
            .returning(move |_, _, _, _| {
                let children = vec![child_a.clone(), child_b.clone()];
                Box::pin(async move { Ok((children, 2)) })
            });

        let deleted_ids = std::sync::Arc::new(std::sync::Mutex::new(Vec::<TaskId>::new()));
        let recorded = deleted_ids.clone();
        task_repository
            .expect_soft_delete()
            .times(3)
            .returning(move |deleted_id, _| {
                recorded.lock().unwrap().push(deleted_id);
                Box::pin(async { Ok(()) })
            });

        let mut service = service(task_repository, MockMemberRepository::new());

        service.soft_delete_task(id).await.unwrap();

        let deleted = deleted_ids.lock().unwrap();
        assert_eq!(deleted.len(), 3);
        assert!(
            deleted.contains(&child_a_id),
            "a dateless (inheriting) child must be deleted too"
        );
        assert!(deleted.contains(&child_b_id));
        assert!(
            deleted.contains(&id),
            "the parent itself must still be deleted"
        );
        // Children first, parent last — matches the order `soft_delete_task`
        // issues the calls in. All of it lands in one transaction either
        // way (see this method's own doc), so this isn't a correctness
        // requirement, just what the implementation actually does.
        assert_eq!(deleted.last(), Some(&id));
    }
}
