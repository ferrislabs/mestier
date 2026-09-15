use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;

use crate::{
    CustomerId, MemberId, OrganizationId, ProjectId, Task, TaskId, TaskRecurrenceId, TaskStatus,
    domain::task::BoardRank, domain::task_label::TaskLabelId,
};

/// Which slice of the hierarchy a listing walks.
///
/// The three values used to be squeezed into one `Option<TaskId>`, where
/// `None` meant "roots only" — fine while `GET /tasks` served exactly one
/// caller, the tree view. A board is the second caller and it wants the
/// opposite default: filtering by project has to return that project's
/// tasks *and its subtasks*, because a subtask is a card like any other and
/// hiding it behind "roots only" would show a column that is quietly
/// missing rows. Two different meanings for `None` in one type is how that
/// goes wrong silently, so the third value is spelled out instead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ParentScope {
    /// Only tasks with no parent. The tree view's opening request, and the
    /// default so that a caller who says nothing gets what `GET /tasks`
    /// has always returned.
    #[default]
    Roots,
    /// Only the direct children of this task — one level, never the whole
    /// subtree.
    ChildrenOf(TaskId),
    /// Every task, roots and subtasks alike, at whatever depth. What a
    /// board query wants: a card's position in the hierarchy says nothing
    /// about which column it belongs in.
    Any,
}

/// Everything `GET /tasks` can narrow a listing by, in one struct.
///
/// A struct rather than a parameter list because the list was about to reach
/// eight positional arguments of which seven are `Option` — a call site
/// nobody can read and which silently accepts two swapped ids of the same
/// type. `Default` is "no narrowing beyond the roots", so a caller names only
/// the axes it cares about.
///
/// Two rules hold across every field, and neither is negotiable:
///
/// * **The organization is not one of these fields.** It is a separate
///   argument on the port method, applied first and always. A filter narrows
///   a set that is already this organization's; a filter is never what keeps
///   another tenant's rows out, because a filter can be absent.
/// * **Filters combine with `AND`**, each one narrowing the previous. The
///   single exception is `statuses`, whose values combine with `OR` among
///   themselves — asking for two columns of a board means the union of the
///   two, never their (empty) intersection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskFilter {
    pub parent: ParentScope,
    /// The project a task is costed against. Independent of the hierarchy:
    /// a subtask may name a project its parent does not, which is why this
    /// pairs with [`ParentScope::Any`] rather than implying anything about
    /// parents.
    pub project_id: Option<ProjectId>,
    /// The columns wanted, combined with `OR`. `None` is "every column";
    /// an empty vector is never constructed — a caller who names no status
    /// is a caller who did not ask for the filter (see
    /// `ListTasksQuery::from_pairs`, which rejects an empty value rather
    /// than turning it into a predicate that matches nothing).
    pub statuses: Option<Vec<TaskStatus>>,
    /// Tasks this member is assigned to. Assignments do not inherit: a task
    /// whose *parent* is assigned to this member is not itself assigned to
    /// them and does not match.
    pub assignee_id: Option<MemberId>,
    pub label_id: Option<TaskLabelId>,
    pub customer_id: Option<CustomerId>,
    /// A case-insensitive substring of the title. Substring, not full-text
    /// search — see `ListTasksQuery::q`.
    pub title_contains: Option<String>,
    /// A window predicate, and *only* a window predicate: `Some(true)` is
    /// `starts_at IS NULL`, `Some(false)` is `starts_at IS NOT NULL`,
    /// `None` does not constrain the window at all. It has nothing to do
    /// with [`TaskStatus::Backlog`] — status and window are independent
    /// columns and stay queryable independently, which is why
    /// `unscheduled=true&status=DONE` is a legitimate question with a
    /// non-empty answer.
    pub unscheduled: Option<bool>,
}

/// Where one card sits on the board.
///
/// The pair travels together because a position is meaningless without its
/// column: two ranks are only comparable inside one `(org_id, status)`, and
/// a drop that named neighbours from two different columns is a drop that
/// brackets nothing. Returning the rank alone is what would let that mistake
/// through silently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardPosition {
    pub status: TaskStatus,
    /// `None` for a card nobody has ever dragged. Not a missing row — see
    /// [`TaskRepository::find_board_positions`] for that distinction.
    pub board_rank: Option<BoardRank>,
}

impl TaskFilter {
    /// Every root task, unnarrowed — what `GET /tasks` with no query string
    /// has always meant.
    pub fn roots() -> Self {
        Self::default()
    }

    /// The direct children of one task, unnarrowed.
    pub fn children_of(parent_task_id: TaskId) -> Self {
        Self {
            parent: ParentScope::ChildrenOf(parent_task_id),
            ..Self::default()
        }
    }
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TaskRepository: Send {
    fn insert(&mut self, task: &Task) -> impl Future<Output = Result<Task, CoreError>> + Send;

    /// Inserts a materialized recurrence occurrence, or does nothing when
    /// its `(recurrence_id, occurrence_date)` is already taken by a
    /// non-deleted row — the arbiter is the partial unique index
    /// `uq_tasks_recurrence_occurrence`. Returns whether a row was actually
    /// inserted, which is what makes a horizon extension idempotent: a retry
    /// or a double claim finds every date already filled and inserts
    /// nothing, rather than erroring or double-booking.
    ///
    /// `task.id` must already be set by the caller (unlike [`Self::insert`],
    /// there is no ambiguity to resolve: on conflict, the id offered is
    /// simply discarded along with the rest of the row).
    fn insert_occurrence_if_absent(
        &mut self,
        task: &Task,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: TaskId,
    ) -> impl Future<Output = Result<Option<Task>, CoreError>> + Send;

    /// A page of `organization_id`'s tasks narrowed by `filter`, plus the
    /// total number of rows the same filter matches (the count is what
    /// pagination metadata is built from, so it is taken under the identical
    /// predicate — a count that ignores a filter reports a page count the
    /// caller can never reach).
    ///
    /// `organization_id` is a separate argument from `filter` on purpose:
    /// the tenant scope is applied first and unconditionally, and no absent
    /// field can widen it. See [`TaskFilter`].
    ///
    /// Ordered `board_rank ASC NULLS LAST, created_at ASC, id ASC` — the
    /// board's own order, served by `idx_tasks_org_id_status_board_rank`.
    /// Ranked cards come first in byte order (the column is `TEXT COLLATE
    /// "C"`), then every card nobody has dragged yet; `created_at` breaks a
    /// tie between two equal ranks and `id` closes the order completely, so
    /// that `LIMIT`/`OFFSET` paging cannot show one row twice or skip
    /// another. `service::sort_by_board_rank` is the in-memory twin of this
    /// clause and the two must keep agreeing.
    ///
    /// Never returns a child's own children unless the filter asks for
    /// [`ParentScope::Any`] — the tree view's read model asks for those with
    /// a second call; see [`Self::count_children`] for how it learns each
    /// root's child count without loading them.
    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        filter: &TaskFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Task>, u64), CoreError>> + Send;

    /// Where each id in `task_ids` currently sits on the board — which
    /// column, and where in it — in one grouped query, scoped to
    /// `organization_id`.
    ///
    /// This is what lets `PATCH /tasks/{id}` take the two cards a drop
    /// landed between instead of a rank computed by the client: the server
    /// reads the neighbours' positions and calls [`BoardRank::between`]
    /// itself, so there is exactly one base-36 implementation in the tree.
    ///
    /// An id absent from the returned map is an id this organization cannot
    /// see — unknown, soft-deleted, or belonging to somebody else — and the
    /// caller turns that into a `404`, never into a rank computed from a row
    /// the caller is not allowed to know exists. The two distinctions the map
    /// keeps are the other ones: which column a neighbour is in (two
    /// neighbours in different columns bracket nothing), and whether it
    /// carries a rank at all (a real card that has never been dragged).
    fn find_board_positions(
        &mut self,
        organization_id: OrganizationId,
        task_ids: &[TaskId],
    ) -> impl Future<Output = Result<HashMap<TaskId, BoardPosition>, CoreError>> + Send;

    /// One whole column — `(organization_id, status)` — in the order the
    /// board displays it: `board_rank ASC NULLS LAST, created_at ASC, id
    /// ASC`, the same clause [`Self::list_by_organization`] uses.
    ///
    /// Unpaginated on purpose, and the only unpaginated read in this port
    /// besides a parent's own children. It feeds
    /// `TaskService::initialize_column`, which has to see the column whole:
    /// the highest rank already in use is its lower bound, and every
    /// unranked card is a row it is about to write. A page would give it
    /// neither.
    ///
    /// Returns ids and ranks rather than tasks — the initialization needs no
    /// other field, and loading assignments for a column nobody is going to
    /// render would be the N+1 this module keeps refusing, in bulk.
    fn list_column_for_ranking(
        &mut self,
        organization_id: OrganizationId,
        status: TaskStatus,
    ) -> impl Future<Output = Result<Vec<(TaskId, Option<BoardRank>)>, CoreError>> + Send;

    /// Writes an initial rank onto each named task, **and only where the row
    /// has none**. Returns how many rows were actually written.
    ///
    /// The `board_rank IS NULL` guard lives in the SQL, not in the caller,
    /// and it is what makes column initialization idempotent and safe under
    /// concurrency at the same time:
    ///
    /// * a second initialization of the same column writes nothing, because
    ///   there is nothing left unranked;
    /// * a card that already carries a rank keeps it, so a half-ranked
    ///   column converges instead of being renumbered — renumbering every
    ///   card on a drop is precisely what fractional ranks exist to avoid;
    /// * two transactions that both decide to initialize the same column
    ///   cannot both succeed. The second blocks on the row locks, then
    ///   re-evaluates the guard against the committed rows and writes zero.
    ///   The returned count is how its caller finds out, and the answer is
    ///   to re-read rather than to trust what it planned to write.
    fn set_board_ranks(
        &mut self,
        organization_id: OrganizationId,
        ranks: &[(TaskId, BoardRank)],
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    /// The tasks assigned to one member that overlap a day, for the field app.
    ///
    /// Deliberately narrow rather than filtering the organization's planning:
    /// an employee's phone must not receive everyone else's schedule, and a
    /// day's worth of one person's jobs is a handful of rows.
    ///
    /// A task with no window is included, since an undated job still has to be
    /// doable; one is excluded only when its window is entirely elsewhere.
    fn list_for_assignee_on(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        day_starts_at: DateTime<Utc>,
        day_ends_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<Task>, CoreError>> + Send;

    /// The number of direct children of each id in `task_ids`, in one
    /// grouped query — never one query per task (see the planning module
    /// design doc's N+1 warning and `GET /tasks`'s contract: it reports each
    /// root's child count without loading the hierarchy). An id with no
    /// children is absent from the map rather than mapped to `0`.
    fn count_children(
        &mut self,
        task_ids: &[TaskId],
    ) -> impl Future<Output = Result<HashMap<TaskId, i64>, CoreError>> + Send;

    /// Persists both the task's own fields and its complete `assignments`
    /// list. The infra adapter replaces assignments as a whole (physical
    /// delete then insert) rather than diffing — matching the `PATCH`
    /// contract, where `assignees` is never a delta.
    fn update(&mut self, task: &Task) -> impl Future<Output = Result<Task, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: TaskId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Soft-deletes every non-deleted occurrence of `recurrence_id` whose
    /// `occurrence_date` is `>= from` — the scope-aware half of a `DELETE`
    /// asking for "this occurrence and every later one", and what
    /// `TaskRecurrenceService::delete_recurrence` uses to remove a deleted
    /// series' future occurrences while leaving its past ones alone.
    /// Returns how many rows were affected.
    fn soft_delete_recurrence_occurrences_from(
        &mut self,
        recurrence_id: TaskRecurrenceId,
        from: NaiveDate,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;
}
