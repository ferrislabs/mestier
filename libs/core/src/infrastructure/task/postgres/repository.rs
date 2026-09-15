use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use mestier_macros::repository;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    MemberId, OrganizationId, Task, TaskAssignment, TaskId, TaskRecurrenceId, TaskStatus,
    domain::task::{
        BoardRank,
        ports::{BoardPosition, ParentScope, TaskFilter, TaskRepository},
    },
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        task::postgres::model::{TaskAssignmentRow, TaskRow},
    },
};

#[repository(domain = Task, backend = Postgres)]
pub struct PgTaskRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgTaskRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> TaskRepository for PgTaskRepository<'tx> {
    async fn insert(&mut self, task: &Task) -> Result<Task, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskRow,
            r#"
            INSERT INTO tasks (id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status, blocks_availability, customer_id, customer_context_id, quote_id, project_id, expenses_cents, expenses_label, board_rank, recurrence_id, occurrence_date, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CAST($9 AS text)::task_status, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            RETURNING id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status::text AS "status!", blocks_availability, customer_id, customer_context_id, quote_id, project_id, expenses_cents, expenses_label, board_rank, recurrence_id, occurrence_date, deleted_at, created_at, updated_at
            "#,
            task.id.0,
            task.organization_id.0,
            task.parent_task_id.map(|id| id.0),
            task.title,
            task.description,
            task.starts_at,
            task.ends_at,
            task.all_day,
            task.status.as_str(),
            task.blocks_availability,
            task.customer_id.map(|id| id.0),
            task.customer_context_id.map(|id| id.0),
            task.quote_id.map(|id| id.0),
            task.project_id.map(|id| id.0),
            task.expenses_cents,
            task.expenses_label,
            task.board_rank.as_ref().map(|rank| rank.0.as_str()),
            task.recurrence_id.map(|id| id.0),
            task.occurrence_date,
            task.deleted_at,
            task.created_at,
            task.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        replace_assignments(&mut tx, task.id, &task.assignments).await?;
        row.into_task(task.assignments.clone())
    }

    async fn insert_occurrence_if_absent(&mut self, task: &Task) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskRow,
            r#"
            INSERT INTO tasks (id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status, blocks_availability, customer_id, customer_context_id, quote_id, project_id, expenses_cents, expenses_label, board_rank, recurrence_id, occurrence_date, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CAST($9 AS text)::task_status, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            ON CONFLICT (recurrence_id, occurrence_date) WHERE recurrence_id IS NOT NULL AND deleted_at IS NULL
            DO NOTHING
            RETURNING id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status::text AS "status!", blocks_availability, customer_id, customer_context_id, quote_id, project_id, expenses_cents, expenses_label, board_rank, recurrence_id, occurrence_date, deleted_at, created_at, updated_at
            "#,
            task.id.0,
            task.organization_id.0,
            task.parent_task_id.map(|id| id.0),
            task.title,
            task.description,
            task.starts_at,
            task.ends_at,
            task.all_day,
            task.status.as_str(),
            task.blocks_availability,
            task.customer_id.map(|id| id.0),
            task.customer_context_id.map(|id| id.0),
            task.quote_id.map(|id| id.0),
            task.project_id.map(|id| id.0),
            task.expenses_cents,
            task.expenses_label,
            task.board_rank.as_ref().map(|rank| rank.0.as_str()),
            task.recurrence_id.map(|id| id.0),
            task.occurrence_date,
            task.deleted_at,
            task.created_at,
            task.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else {
            return Ok(false);
        };

        replace_assignments(&mut tx, TaskId(row.id), &task.assignments).await?;
        Ok(true)
    }

    async fn find_by_id(&mut self, id: TaskId) -> Result<Option<Task>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskRow,
            r#"
            SELECT id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status::text AS "status!", blocks_availability, customer_id, customer_context_id, quote_id, project_id, expenses_cents, expenses_label, board_rank, recurrence_id, occurrence_date, deleted_at, created_at, updated_at
            FROM tasks
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(row) => {
                let assignments = fetch_assignments(&mut tx, id).await?;
                Ok(Some(row.into_task(assignments)?))
            }
            None => Ok(None),
        }
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        filter: &TaskFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Task>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let FilterParams {
            any_parent,
            parent_task_id,
            project_id,
            statuses,
            assignee_id,
            label_id,
            customer_id,
            title_contains,
            unscheduled,
        } = FilterParams::from(filter);

        // The predicate below and the count's are one predicate written
        // twice, and they have to stay that way: a count taken under a wider
        // `WHERE` reports pages the caller can never reach, and one taken
        // under a narrower one hides the tail of the result. `query_as!`
        // needs its SQL as a literal, so the duplication cannot be factored
        // out into a shared fragment — changing one means changing both.
        //
        // Every `$n::type IS NULL OR ...` is the same shape: an absent filter
        // contributes a predicate that is true for every row, so filters
        // compose with `AND` and an absent one narrows nothing. The org scope
        // is not written that way on purpose — it has no `IS NULL` escape
        // hatch, because it is never optional.
        let rows = sqlx::query_as!(
            TaskRow,
            r#"
            SELECT t.id, t.org_id, t.parent_task_id, t.title, t.description, t.starts_at, t.ends_at, t.all_day, t.status::text AS "status!", t.blocks_availability, t.customer_id, t.customer_context_id, t.quote_id, t.project_id, t.expenses_cents, t.expenses_label, t.board_rank, t.recurrence_id, t.occurrence_date, t.deleted_at, t.created_at, t.updated_at
            FROM tasks t
            WHERE t.org_id = $1 AND t.deleted_at IS NULL
              AND (
                $2::boolean
                OR ($3::uuid IS NULL AND t.parent_task_id IS NULL)
                OR t.parent_task_id = $3
              )
              AND ($4::uuid IS NULL OR t.project_id = $4)
              AND ($5::text[] IS NULL OR t.status::text = ANY($5))
              AND (
                $6::uuid IS NULL
                OR EXISTS (
                  SELECT 1 FROM task_assignments a
                  WHERE a.task_id = t.id AND a.member_id = $6 AND a.org_id = t.org_id
                )
              )
              AND (
                $7::uuid IS NULL
                OR EXISTS (
                  SELECT 1 FROM task_label_links l
                  WHERE l.task_id = t.id AND l.label_id = $7
                )
              )
              AND ($8::uuid IS NULL OR t.customer_id = $8)
              AND ($9::text IS NULL OR POSITION(LOWER($9) IN LOWER(t.title)) > 0)
              AND ($10::boolean IS NULL OR (t.starts_at IS NULL) = $10)
            ORDER BY t.board_rank ASC NULLS LAST, t.created_at ASC, t.id ASC
            LIMIT $11 OFFSET $12
            "#,
            organization_id.0,
            any_parent,
            parent_task_id,
            project_id,
            statuses.as_deref(),
            assignee_id,
            label_id,
            customer_id,
            title_contains,
            unscheduled,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM tasks t
            WHERE t.org_id = $1 AND t.deleted_at IS NULL
              AND (
                $2::boolean
                OR ($3::uuid IS NULL AND t.parent_task_id IS NULL)
                OR t.parent_task_id = $3
              )
              AND ($4::uuid IS NULL OR t.project_id = $4)
              AND ($5::text[] IS NULL OR t.status::text = ANY($5))
              AND (
                $6::uuid IS NULL
                OR EXISTS (
                  SELECT 1 FROM task_assignments a
                  WHERE a.task_id = t.id AND a.member_id = $6 AND a.org_id = t.org_id
                )
              )
              AND (
                $7::uuid IS NULL
                OR EXISTS (
                  SELECT 1 FROM task_label_links l
                  WHERE l.task_id = t.id AND l.label_id = $7
                )
              )
              AND ($8::uuid IS NULL OR t.customer_id = $8)
              AND ($9::text IS NULL OR POSITION(LOWER($9) IN LOWER(t.title)) > 0)
              AND ($10::boolean IS NULL OR (t.starts_at IS NULL) = $10)
            "#,
            organization_id.0,
            any_parent,
            parent_task_id,
            project_id,
            statuses.as_deref(),
            assignee_id,
            label_id,
            customer_id,
            title_contains,
            unscheduled,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let mut assignments_by_task = fetch_assignments_for_tasks(&mut tx, &ids).await?;

        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            let assignments = assignments_by_task.remove(&row.id).unwrap_or_default();
            tasks.push(row.into_task(assignments)?);
        }

        Ok((tasks, total as u64))
    }

    async fn find_board_positions(
        &mut self,
        organization_id: OrganizationId,
        task_ids: &[TaskId],
    ) -> Result<HashMap<TaskId, BoardPosition>, CoreError> {
        if task_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut tx = self.tx.lock().await;
        let ids: Vec<Uuid> = task_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query!(
            r#"
            SELECT id, status::text AS "status!", board_rank
            FROM tasks
            WHERE org_id = $1 AND deleted_at IS NULL AND id = ANY($2)
            "#,
            organization_id.0,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        // A row that did not come back is simply absent from the map. The
        // `org_id` predicate is what makes that the same answer for "no such
        // task" and "somebody else's task" — the caller cannot tell them
        // apart, which is the point.
        rows.into_iter()
            .map(|row| {
                let status = TaskStatus::from_str(&row.status).map_err(|error| {
                    CoreError::Internal(format!("invalid task status in database: {error}"))
                })?;
                Ok((
                    TaskId(row.id),
                    BoardPosition {
                        status,
                        board_rank: row.board_rank.map(BoardRank),
                    },
                ))
            })
            .collect()
    }

    async fn list_column_for_ranking(
        &mut self,
        organization_id: OrganizationId,
        status: TaskStatus,
    ) -> Result<Vec<(TaskId, Option<BoardRank>)>, CoreError> {
        let mut tx = self.tx.lock().await;
        // The same ordering clause as `list_by_organization`, and it has to
        // be: this is the order the column is being displayed in, and the
        // order initialization is about to write down. If the two drifted,
        // materializing a column would silently reshuffle it.
        let rows = sqlx::query!(
            r#"
            SELECT id, board_rank
            FROM tasks
            WHERE org_id = $1 AND deleted_at IS NULL AND status = CAST($2 AS text)::task_status
            ORDER BY board_rank ASC NULLS LAST, created_at ASC, id ASC
            "#,
            organization_id.0,
            status.as_str(),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| (TaskId(row.id), row.board_rank.map(BoardRank)))
            .collect())
    }

    async fn set_board_ranks(
        &mut self,
        organization_id: OrganizationId,
        ranks: &[(TaskId, BoardRank)],
    ) -> Result<u64, CoreError> {
        if ranks.is_empty() {
            return Ok(0);
        }

        let mut tx = self.tx.lock().await;
        let ids: Vec<Uuid> = ranks.iter().map(|(id, _)| id.0).collect();
        let values: Vec<String> = ranks.iter().map(|(_, rank)| rank.0.clone()).collect();

        // One statement for the whole column rather than one per card: an
        // initialization touches every unranked row at once, and issuing it
        // row by row would be the N+1 this module refuses everywhere else,
        // at its worst — on a write.
        //
        // `board_rank IS NULL` is the guard the port's contract is written
        // around. It is in the SQL rather than in the caller because that is
        // the only place it is also a concurrency guarantee: a second
        // transaction reaching this statement blocks on the row locks, then
        // re-evaluates the predicate against the committed rows and updates
        // nothing.
        let affected = sqlx::query!(
            r#"
            UPDATE tasks AS t
            SET board_rank = incoming.board_rank, updated_at = now()
            FROM UNNEST($2::uuid[], $3::text[]) AS incoming(id, board_rank)
            WHERE t.id = incoming.id
              AND t.org_id = $1
              AND t.deleted_at IS NULL
              AND t.board_rank IS NULL
            "#,
            organization_id.0,
            &ids,
            &values,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(affected)
    }

    async fn list_for_assignee_on(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        day_starts_at: DateTime<Utc>,
        day_ends_at: DateTime<Utc>,
    ) -> Result<Vec<Task>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            TaskRow,
            r#"
            SELECT t.id, t.org_id, t.parent_task_id, t.title, t.description, t.starts_at, t.ends_at, t.all_day, t.status::text AS "status!", t.blocks_availability, t.customer_id, t.customer_context_id, t.quote_id, t.project_id, t.expenses_cents, t.expenses_label, t.board_rank, t.recurrence_id, t.occurrence_date, t.deleted_at, t.created_at, t.updated_at
            FROM tasks t
            JOIN task_assignments a ON a.task_id = t.id
            WHERE t.org_id = $1
              AND a.member_id = $2
              AND t.deleted_at IS NULL
              AND (
                t.starts_at IS NULL
                OR (t.starts_at < $4 AND COALESCE(t.ends_at, t.starts_at) >= $3)
              )
            ORDER BY t.starts_at ASC NULLS LAST, t.id ASC
            "#,
            organization_id.0,
            member_id.0,
            day_starts_at,
            day_ends_at,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let mut assignments_by_task = fetch_assignments_for_tasks(&mut tx, &ids).await?;

        rows.into_iter()
            .map(|row| {
                let assignments = assignments_by_task.remove(&row.id).unwrap_or_default();
                row.into_task(assignments)
            })
            .collect()
    }

    async fn count_children(
        &mut self,
        task_ids: &[TaskId],
    ) -> Result<HashMap<TaskId, i64>, CoreError> {
        if task_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut tx = self.tx.lock().await;
        let ids: Vec<Uuid> = task_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query!(
            r#"
            SELECT parent_task_id AS "parent_task_id!", COUNT(*) AS "count!"
            FROM tasks
            WHERE parent_task_id = ANY($1) AND deleted_at IS NULL
            GROUP BY parent_task_id
            "#,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| (TaskId(row.parent_task_id), row.count))
            .collect())
    }

    async fn update(&mut self, task: &Task) -> Result<Task, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskRow,
            r#"
            UPDATE tasks
            SET parent_task_id = $2,
                title = $3,
                description = $4,
                starts_at = $5,
                ends_at = $6,
                all_day = $7,
                status = CAST($8 AS text)::task_status,
                blocks_availability = $9,
                updated_at = $10,
                project_id = $11,
                expenses_cents = $12,
                expenses_label = $13,
                recurrence_id = $14,
                board_rank = $15
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status::text AS "status!", blocks_availability, customer_id, customer_context_id, quote_id, project_id, expenses_cents, expenses_label, board_rank, recurrence_id, occurrence_date, deleted_at, created_at, updated_at
            "#,
            task.id.0,
            task.parent_task_id.map(|id| id.0),
            task.title,
            task.description,
            task.starts_at,
            task.ends_at,
            task.all_day,
            task.status.as_str(),
            task.blocks_availability,
            task.updated_at,
            task.project_id.map(|id| id.0),
            task.expenses_cents,
            task.expenses_label,
            task.recurrence_id.map(|id| id.0),
            task.board_rank.as_ref().map(|rank| rank.0.as_str()),
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;

        replace_assignments(&mut tx, task.id, &task.assignments).await?;
        row.into_task(task.assignments.clone())
    }

    async fn soft_delete(
        &mut self,
        id: TaskId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE tasks
            SET deleted_at = $2, updated_at = $2
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn soft_delete_recurrence_occurrences_from(
        &mut self,
        recurrence_id: TaskRecurrenceId,
        from: NaiveDate,
        deleted_at: DateTime<Utc>,
    ) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE tasks
            SET deleted_at = $3, updated_at = $3
            WHERE recurrence_id = $1 AND occurrence_date >= $2 AND deleted_at IS NULL
            "#,
            recurrence_id.0,
            from,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }
}

async fn fetch_assignments(
    conn: &mut PgConnection,
    task_id: TaskId,
) -> Result<Vec<TaskAssignment>, CoreError> {
    let rows = sqlx::query_as!(
        TaskAssignmentRow,
        r#"
        SELECT id, org_id, task_id, member_id, created_at
        FROM task_assignments
        WHERE task_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
        task_id.0,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Every assignment for each id in `task_ids`, in one grouped query — never
/// one per task. Mirrors `count_children`'s own batching (`WHERE ... = ANY($1)`
/// plus in-memory grouping); `list_by_organization` used to call
/// `fetch_assignments` once per row inside a loop, a pre-existing N+1 found
/// while proving `GET /tasks`'s total query count for the task-labels
/// workstream (#142) — fixed here rather than left for a separate PR, since
/// T5's list view (#145) is about to hit this same endpoint hard. A task
/// with no assignments is absent from the map rather than mapped to an
/// empty `Vec`, same contract as `count_children`.
async fn fetch_assignments_for_tasks(
    conn: &mut PgConnection,
    task_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<TaskAssignment>>, CoreError> {
    if task_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as!(
        TaskAssignmentRow,
        r#"
        SELECT id, org_id, task_id, member_id, created_at
        FROM task_assignments
        WHERE task_id = ANY($1)
        ORDER BY task_id ASC, created_at ASC, id ASC
        "#,
        task_ids,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;

    let mut grouped: HashMap<Uuid, Vec<TaskAssignment>> = HashMap::new();
    for row in rows {
        grouped.entry(row.task_id).or_default().push(row.into());
    }

    Ok(grouped)
}

/// Replaces the complete assignment set for a task: physical delete of
/// whatever is there, then insert of the new list. Assignments carry no
/// history worth keeping once removed (unlike tasks themselves, which are
/// soft-deleted), and the `PATCH` contract treats `assignees` as the full
/// list rather than a delta, so a diff would add complexity without adding
/// correctness.
async fn replace_assignments(
    conn: &mut PgConnection,
    task_id: TaskId,
    assignments: &[TaskAssignment],
) -> Result<(), CoreError> {
    sqlx::query!(
        r#"DELETE FROM task_assignments WHERE task_id = $1"#,
        task_id.0,
    )
    .execute(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;

    for assignment in assignments {
        sqlx::query!(
            r#"
            INSERT INTO task_assignments (id, org_id, task_id, member_id, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            assignment.id.0,
            assignment.organization_id.0,
            assignment.task_id.0,
            assignment.member_id.0,
            assignment.created_at,
        )
        .execute(&mut *conn)
        .await
        .map_err(map_sqlx_error)?;
    }

    Ok(())
}

/// The filter, flattened into the scalar bind parameters `query_as!` needs.
///
/// Built once and destructured into both statements below so the list and its
/// count cannot drift apart in the binding order — the one mistake this
/// duplicated `WHERE` invites, and one the type checker would not catch
/// between two `Option<Uuid>`s.
struct FilterParams {
    /// [`ParentScope::Any`] collapsed to a plain flag, because SQL has no
    /// three-valued parameter and `NULL` already means "roots" here.
    any_parent: bool,
    parent_task_id: Option<Uuid>,
    project_id: Option<Uuid>,
    /// The statuses as their wire strings: the column is the `task_status`
    /// enum, and comparing `status::text = ANY($n)` against a `text[]` keeps
    /// the binding plain rather than teaching sqlx an array of a custom enum
    /// type. `TaskStatus::as_str` is the same spelling the column stores.
    statuses: Option<Vec<String>>,
    assignee_id: Option<Uuid>,
    label_id: Option<Uuid>,
    customer_id: Option<Uuid>,
    title_contains: Option<String>,
    unscheduled: Option<bool>,
}

impl From<&TaskFilter> for FilterParams {
    fn from(filter: &TaskFilter) -> Self {
        let (any_parent, parent_task_id) = match filter.parent {
            ParentScope::Roots => (false, None),
            ParentScope::ChildrenOf(id) => (false, Some(id.0)),
            ParentScope::Any => (true, None),
        };

        Self {
            any_parent,
            parent_task_id,
            project_id: filter.project_id.map(|id| id.0),
            statuses: filter.statuses.as_ref().map(|statuses| {
                statuses
                    .iter()
                    .map(|status| status.as_str().to_owned())
                    .collect()
            }),
            assignee_id: filter.assignee_id.map(|id| id.0),
            label_id: filter.label_id.map(|id| id.0),
            customer_id: filter.customer_id.map(|id| id.0),
            title_contains: filter.title_contains.clone(),
            unscheduled: filter.unscheduled,
        }
    }
}
