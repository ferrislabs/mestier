use std::str::FromStr;

use auth::Identity;
use axum::{
    Extension,
    extract::{FromRequestParts, Query, State},
    http::request::Parts,
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{
    CustomerId, MemberId, ProjectId, TaskId, TaskLabelId, TaskStatus,
    application::task::{ParentScope, TaskFilter},
};
use utoipa::IntoParams;

use crate::{require_org_membership, response::TaskResponse, task::TasksPath};

/// Everything `GET /tasks` can be narrowed by. Filters combine with `AND`;
/// the values of a repeated `status` combine with `OR` among themselves.
///
/// ## Wire form
///
/// `?project_id=<uuid>&status=BACKLOG&status=IN_PROGRESS&q=toiture&unscheduled=true`
///
/// `status` is the only repeatable key, and it accepts **either** form:
/// repeated (`status=BACKLOG&status=IN_PROGRESS`) or comma-separated
/// (`status=BACKLOG,IN_PROGRESS`). Both mean the union of those columns. The
/// repeated form is the one a generated client should emit, since it needs no
/// agreement about a separator; the comma form exists because hand-written
/// URLs and `curl` reach for it first.
///
/// Both forms are parsed here, by hand, rather than by `serde`: axum's own
/// `Query<T>` goes through `serde_urlencoded`, which keeps a single value per
/// key, so `status=BACKLOG&status=IN_PROGRESS` never reaches a
/// `Vec<TaskStatus>` intact. This extractor asks for the decoded key/value
/// pairs instead — `Query<Vec<(String, String)>>`, which `serde_urlencoded`
/// *does* produce for a repeated key — and assembles the struct from them.
///
/// ## Nothing is ignored
///
/// A value that does not parse is a `400`, and so is a repeat of a key that
/// is not repeatable, and so is a key this endpoint does not know. None of
/// those can be answered with "filter absent": a filter that quietly does
/// nothing returns wrong data that looks right, and a typo in a parameter
/// name is exactly how that happens in the field.
#[derive(Debug, Default, PartialEq, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListTasksQuery {
    /// Scopes the listing to one task's direct children. Absent, the
    /// listing covers the roots — unless another filter is present, in
    /// which case it covers every task at any depth; see
    /// [`ListTasksQuery::parent_scope`].
    pub parent_task_id: Option<TaskId>,
    /// The project a task is costed against. Matches roots and subtasks
    /// alike: a subtask is a card like any other, and a project's board that
    /// hides them is a board that is quietly missing rows.
    pub project_id: Option<ProjectId>,
    /// One or more board columns, combined with `OR`. See the wire form
    /// above for the two spellings.
    pub status: Option<Vec<TaskStatus>>,
    /// Tasks assigned to this member. Assignments do not inherit — a task
    /// whose *parent* is assigned to them does not match.
    pub assignee_id: Option<MemberId>,
    pub label_id: Option<TaskLabelId>,
    pub customer_id: Option<CustomerId>,
    /// A case-insensitive **substring** of the title. Not full-text search:
    /// there is no `tsvector`, no stemming, no ranking and no word
    /// boundaries, so `oitu` matches `Réfection toiture`. Reach for
    /// PostgreSQL's text search when the product actually asks for
    /// relevance, and change this doc comment when you do.
    ///
    /// `?q=` (an empty value) matches every title, because the empty string
    /// is a substring of every string. That is the right answer rather than
    /// a dropped filter — it is what a cleared search box means — and it is
    /// the one place where a present filter narrows nothing.
    pub q: Option<String>,
    /// A window predicate and nothing else: `true` is `starts_at IS NULL`,
    /// `false` is `starts_at IS NOT NULL`. It is **not** a synonym for
    /// `status=BACKLOG` — status and window are independent columns, so
    /// `?unscheduled=true&status=DONE` is a legitimate question ("what did
    /// we finish that was never on the calendar?") with a non-empty answer.
    pub unscheduled: Option<bool>,
}

/// The pagination keys, which share the query string with the filters and
/// are parsed by their own extractor. Listed here so that the unknown-key
/// check below does not reject them.
const PAGINATION_KEYS: [&str; 2] = ["page", "per_page"];

impl ListTasksQuery {
    /// Assembles the filter set from the decoded query-string pairs, in
    /// order, rejecting anything it cannot account for.
    ///
    /// Pure, and separate from the extractor, so every rule above is
    /// testable without a request: the repeated key, the comma form, the
    /// unparsable value, the unknown key.
    fn from_pairs(pairs: &[(String, String)]) -> Result<Self, ApiError> {
        let mut query = Self::default();
        let mut statuses: Vec<TaskStatus> = Vec::new();
        let mut saw_status = false;

        for (key, raw) in pairs {
            match key.as_str() {
                "parent_task_id" => set_once(&mut query.parent_task_id, key, parse(key, raw)?)?,
                "project_id" => set_once(&mut query.project_id, key, parse(key, raw)?)?,
                "assignee_id" => set_once(&mut query.assignee_id, key, parse(key, raw)?)?,
                "label_id" => set_once(&mut query.label_id, key, parse(key, raw)?)?,
                "customer_id" => set_once(&mut query.customer_id, key, parse(key, raw)?)?,
                "unscheduled" => set_once(&mut query.unscheduled, key, parse_bool(key, raw)?)?,
                "q" => set_once(&mut query.q, key, raw.clone())?,
                "status" => {
                    saw_status = true;
                    // The comma form and the repeated form land in the same
                    // vector, so `status=A,B&status=C` is the union of the
                    // three rather than an error about mixing spellings.
                    for value in raw.split(',') {
                        statuses.push(parse_status(value)?);
                    }
                }
                other if PAGINATION_KEYS.contains(&other) => {}
                other => {
                    return Err(ApiError::BadRequest(format!(
                        "unknown query parameter `{other}`"
                    )));
                }
            }
        }

        if saw_status {
            if statuses.is_empty() {
                // Only reachable as `?status=`, which is a caller who meant
                // to name a column and named none. Answering it with "no
                // status filter" would widen the result instead of narrowing
                // it — the opposite of what they asked for.
                return Err(ApiError::BadRequest(
                    "`status` was given no value; expected one of BACKLOG, PLANNED, IN_PROGRESS, \
                     DONE, CANCELLED"
                        .to_owned(),
                ));
            }
            // Kept verbatim, duplicates included: the SQL side is `status =
            // ANY($n)`, for which a repeated value changes nothing, and
            // rewriting what the caller sent would only make the 400 messages
            // above harder to trace back to the URL that produced them.
            query.status = Some(statuses);
        }

        Ok(query)
    }

    /// How far down the hierarchy the listing reaches.
    ///
    /// Naming a parent is unambiguous: that task's direct children. Naming
    /// none is the interesting case, and it depends on what else was asked:
    ///
    /// * no filter at all — the roots, which is what `GET /tasks` has always
    ///   returned and what the tree view opens with, each row carrying its
    ///   `child_count`;
    /// * any filter — every task at any depth. A board asking for a
    ///   project's `IN_PROGRESS` cards wants the subtasks too; restricting a
    ///   filtered query to roots would answer with a column that silently
    ///   omits them, which is worse than answering nothing.
    fn parent_scope(&self) -> ParentScope {
        match self.parent_task_id {
            Some(id) => ParentScope::ChildrenOf(id),
            None if self.narrows_beyond_the_hierarchy() => ParentScope::Any,
            None => ParentScope::Roots,
        }
    }

    fn narrows_beyond_the_hierarchy(&self) -> bool {
        self.project_id.is_some()
            || self.status.is_some()
            || self.assignee_id.is_some()
            || self.label_id.is_some()
            || self.customer_id.is_some()
            || self.q.is_some()
            || self.unscheduled.is_some()
    }
}

impl From<ListTasksQuery> for TaskFilter {
    fn from(query: ListTasksQuery) -> Self {
        let parent = query.parent_scope();
        Self {
            parent,
            project_id: query.project_id,
            statuses: query.status,
            assignee_id: query.assignee_id,
            label_id: query.label_id,
            customer_id: query.customer_id,
            title_contains: query.q,
            unscheduled: query.unscheduled,
        }
    }
}

fn parse<T: FromStr>(key: &str, raw: &str) -> Result<T, ApiError> {
    T::from_str(raw).map_err(|_| ApiError::BadRequest(format!("invalid `{key}`: `{raw}`")))
}

/// `true`/`false` only. `serde` would also take `1`/`0`/`on`; one spelling
/// keeps the generated client and the hand-written URL saying the same thing.
fn parse_bool(key: &str, raw: &str) -> Result<bool, ApiError> {
    match raw {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(ApiError::BadRequest(format!(
            "invalid `{key}`: `{other}`; expected `true` or `false`"
        ))),
    }
}

/// The wire spelling of [`TaskStatus`], which is also what the `task_status`
/// enum stores — `TaskStatus::as_str` is the single source of both.
fn parse_status(raw: &str) -> Result<TaskStatus, ApiError> {
    [
        TaskStatus::Backlog,
        TaskStatus::Planned,
        TaskStatus::InProgress,
        TaskStatus::Done,
        TaskStatus::Cancelled,
    ]
    .into_iter()
    .find(|status| status.as_str() == raw)
    .ok_or_else(|| {
        ApiError::BadRequest(format!(
            "invalid `status`: `{raw}`; expected one of BACKLOG, PLANNED, IN_PROGRESS, DONE, \
             CANCELLED"
        ))
    })
}

/// Refuses a second value for a key that carries one. Last-one-wins is the
/// alternative, and it silently drops half of what the caller asked for.
fn set_once<T>(slot: &mut Option<T>, key: &str, value: T) -> Result<(), ApiError> {
    if slot.is_some() {
        return Err(ApiError::BadRequest(format!(
            "`{key}` was given more than once; only `status` is repeatable"
        )));
    }
    *slot = Some(value);
    Ok(())
}

impl<S> FromRequestParts<S> for ListTasksQuery
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // `Vec<(String, String)>` rather than a struct: this is the one shape
        // `serde_urlencoded` deserializes without collapsing a repeated key,
        // and it arrives percent-decoded, so `q=r%C3%A9fection` is already
        // `réfection` by the time it gets here.
        let Query(pairs) = Query::<Vec<(String, String)>>::from_request_parts(parts, state)
            .await
            .map_err(|rejection| ApiError::BadRequest(rejection.to_string()))?;

        Self::from_pairs(&pairs)
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/tasks",
    operation_id = "listTasks",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
        ListTasksQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of tasks, ordered by board rank with unranked tasks last — each root's child_count is populated, computed without loading its subtasks", body = inline(DataEnvelope<Vec<TaskResponse>>)),
        (status = 400, description = "Unknown, repeated or unparsable query parameter"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TasksPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
    filter: ListTasksQuery,
) -> Result<Response<TaskResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    // `organization_id` travels beside the filter, never inside it: the
    // tenant scope is applied first and is not something an absent field can
    // widen.
    let (tasks, child_counts, total) = state
        .usecase
        .list_tasks(path.organization_id, filter.into(), per_page, offset)
        .await?;

    // Still two grouped queries for the whole page, whatever the filters
    // narrowed it to — one for labels, one for equipment. Never one per task;
    // see the planning module design doc's N+1 warning.
    let task_ids: Vec<TaskId> = tasks.iter().map(|task| task.id).collect();
    let mut labels_by_task = state
        .usecase
        .list_task_labels_for_tasks(task_ids.clone())
        .await?;
    let mut equipment_by_task = state.usecase.list_equipment_for_tasks(task_ids).await?;

    let items: Vec<TaskResponse> = tasks
        .into_iter()
        .map(|task| {
            let child_count = child_counts.get(&task.id).copied().unwrap_or(0);
            let labels = labels_by_task.remove(&task.id).unwrap_or_default();
            let equipment = equipment_by_task.remove(&task.id).unwrap_or_default();
            TaskResponse {
                child_count: Some(child_count),
                labels: labels.into_iter().map(Into::into).collect(),
                equipment: equipment.into_iter().map(Into::into).collect(),
                ..TaskResponse::from(task)
            }
        })
        .collect();
    let is_empty = items.is_empty();
    // `total` is counted under the same predicate the page was selected with,
    // so the metadata stays right whatever combination of filters was asked
    // for — see `TaskRepository::list_by_organization`.
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real decoding path, not a stand-in: `Query::try_from_uri` is
    /// exactly what the extractor above runs, `serde_urlencoded` included.
    /// So these tests also pin the claim the whole hand-written extractor
    /// rests on — that a repeated key survives as two entries in a
    /// `Vec<(String, String)>` where it would have collapsed into one field
    /// of a struct.
    fn pairs(raw: &str) -> Vec<(String, String)> {
        let uri: axum::http::Uri = format!("/tasks?{raw}").parse().unwrap();
        let Query(pairs) = Query::<Vec<(String, String)>>::try_from_uri(&uri)
            .expect("the query string must decode into key/value pairs");
        pairs
    }

    fn parse_query(raw: &str) -> Result<ListTasksQuery, ApiError> {
        ListTasksQuery::from_pairs(&pairs(raw))
    }

    /// The load-bearing assumption, stated on its own so that a future
    /// change of extractor fails here first and with a legible message.
    #[test]
    fn a_repeated_key_survives_url_decoding_as_two_pairs() {
        assert_eq!(
            pairs("status=BACKLOG&status=IN_PROGRESS"),
            vec![
                ("status".to_owned(), "BACKLOG".to_owned()),
                ("status".to_owned(), "IN_PROGRESS".to_owned()),
            ]
        );
    }

    /// Percent-decoding is `serde_urlencoded`'s job, and the extractor
    /// relies on it having already happened: `q` reaches the SQL layer as
    /// text, never as an escape sequence.
    #[test]
    fn a_percent_encoded_value_is_decoded_before_it_is_parsed() {
        let query = parse_query("q=r%C3%A9fection+toiture").unwrap();

        assert_eq!(query.q, Some("réfection toiture".to_owned()));
    }

    #[test]
    fn an_empty_query_string_filters_nothing_and_lists_the_roots() {
        let query = parse_query("").expect("an empty query must parse");

        assert_eq!(query, ListTasksQuery::default());
        assert_eq!(
            query.parent_scope(),
            ParentScope::Roots,
            "`GET /tasks` with no query string must keep returning the roots"
        );
    }

    /// The reason this extractor is hand-written: `serde_urlencoded` keeps
    /// one value per key, so a derived `Query<ListTasksQuery>` would see one
    /// status here, not two. Two is what the board asks for.
    #[test]
    fn a_repeated_status_carries_both_values() {
        let query = parse_query("status=BACKLOG&status=IN_PROGRESS").unwrap();

        assert_eq!(
            query.status,
            Some(vec![TaskStatus::Backlog, TaskStatus::InProgress]),
            "`status=BACKLOG&status=IN_PROGRESS` must mean the union of the two columns, not \
             whichever value happened to be written last"
        );
    }

    #[test]
    fn a_comma_separated_status_carries_both_values_too() {
        let query = parse_query("status=BACKLOG,IN_PROGRESS").unwrap();

        assert_eq!(
            query.status,
            Some(vec![TaskStatus::Backlog, TaskStatus::InProgress])
        );
    }

    #[test]
    fn the_two_status_spellings_mix() {
        let query = parse_query("status=BACKLOG,PLANNED&status=DONE").unwrap();

        assert_eq!(
            query.status,
            Some(vec![
                TaskStatus::Backlog,
                TaskStatus::Planned,
                TaskStatus::Done
            ])
        );
    }

    #[test]
    fn an_unknown_status_is_rejected_rather_than_dropped() {
        let error = parse_query("status=BACKLOG&status=ALMOST").unwrap_err();

        assert!(
            matches!(error, ApiError::BadRequest(ref message) if message.contains("ALMOST")),
            "an unparsable status must be a 400, never a silently narrower filter: {error:?}"
        );
    }

    #[test]
    fn an_empty_status_is_rejected_rather_than_treated_as_no_filter() {
        let error = parse_query("status=").unwrap_err();

        assert!(matches!(error, ApiError::BadRequest(_)));
    }

    #[test]
    fn a_malformed_project_id_is_a_bad_request() {
        let error = parse_query("project_id=not-a-uuid").unwrap_err();

        assert!(
            matches!(error, ApiError::BadRequest(ref message) if message.contains("project_id")),
            "{error:?}"
        );
    }

    /// Last-one-wins on a single-valued key would answer a question the
    /// caller did not ask, using half of what they sent.
    #[test]
    fn a_repeated_single_valued_key_is_rejected() {
        let a = "11111111-1111-1111-1111-111111111111";
        let b = "22222222-2222-2222-2222-222222222222";
        let error = parse_query(&format!("project_id={a}&project_id={b}")).unwrap_err();

        assert!(matches!(error, ApiError::BadRequest(_)));
    }

    /// The typo case. `projet_id` is not a filter this endpoint has, and
    /// accepting it would return every task in the organization while
    /// looking exactly like a successful filtered query.
    #[test]
    fn an_unknown_parameter_is_rejected() {
        let error = parse_query("projet_id=11111111-1111-1111-1111-111111111111").unwrap_err();

        assert!(
            matches!(error, ApiError::BadRequest(ref message) if message.contains("projet_id")),
            "{error:?}"
        );
    }

    #[test]
    fn pagination_keys_are_not_mistaken_for_unknown_filters() {
        let query = parse_query("page=2&per_page=50").expect("pagination keys must be tolerated");

        assert_eq!(query, ListTasksQuery::default());
    }

    #[test]
    fn unscheduled_takes_true_and_false_and_nothing_else() {
        assert_eq!(
            parse_query("unscheduled=true").unwrap().unscheduled,
            Some(true)
        );
        assert_eq!(
            parse_query("unscheduled=false").unwrap().unscheduled,
            Some(false)
        );
        assert!(matches!(
            parse_query("unscheduled=1").unwrap_err(),
            ApiError::BadRequest(_)
        ));
    }

    /// The independence rule at the parsing layer: the two keys are read
    /// into two unrelated fields, neither deriving the other.
    #[test]
    fn unscheduled_and_status_are_two_independent_filters() {
        let query = parse_query("unscheduled=true&status=DONE").unwrap();

        assert_eq!(query.unscheduled, Some(true));
        assert_eq!(query.status, Some(vec![TaskStatus::Done]));

        let filter = TaskFilter::from(query);
        assert_eq!(filter.unscheduled, Some(true));
        assert_eq!(filter.statuses, Some(vec![TaskStatus::Done]));
    }

    /// A project's board shows its subtasks. Restricting a filtered listing
    /// to roots would drop them without saying so.
    #[test]
    fn a_filtered_listing_reaches_subtasks_while_an_unfiltered_one_lists_roots() {
        let project_id: ProjectId = "33333333-3333-3333-3333-333333333333".parse().unwrap();

        let filtered = parse_query(&format!("project_id={project_id}")).unwrap();
        assert_eq!(filtered.parent_scope(), ParentScope::Any);

        let unfiltered = parse_query("").unwrap();
        assert_eq!(unfiltered.parent_scope(), ParentScope::Roots);
    }

    /// Naming a parent is never widened by another filter: the caller asked
    /// for that task's children and gets exactly those, narrowed further.
    #[test]
    fn naming_a_parent_wins_over_the_widening_rule() {
        let parent: TaskId = "44444444-4444-4444-4444-444444444444".parse().unwrap();
        let query = parse_query(&format!("parent_task_id={parent}&status=DONE")).unwrap();

        assert_eq!(query.parent_scope(), ParentScope::ChildrenOf(parent));
    }

    #[test]
    fn every_filter_reaches_the_task_filter_it_maps_to() {
        let project_id: ProjectId = "33333333-3333-3333-3333-333333333333".parse().unwrap();
        let assignee_id: MemberId = "55555555-5555-5555-5555-555555555555".parse().unwrap();
        let label_id: TaskLabelId = "66666666-6666-6666-6666-666666666666".parse().unwrap();
        let customer_id: CustomerId = "77777777-7777-7777-7777-777777777777".parse().unwrap();

        let query = parse_query(&format!(
            "project_id={project_id}&assignee_id={assignee_id}&label_id={label_id}\
             &customer_id={customer_id}&q=toiture&status=BACKLOG&unscheduled=false"
        ))
        .unwrap();
        let filter = TaskFilter::from(query);

        assert_eq!(filter.project_id, Some(project_id));
        assert_eq!(filter.assignee_id, Some(assignee_id));
        assert_eq!(filter.label_id, Some(label_id));
        assert_eq!(filter.customer_id, Some(customer_id));
        assert_eq!(filter.title_contains, Some("toiture".to_owned()));
        assert_eq!(filter.statuses, Some(vec![TaskStatus::Backlog]));
        assert_eq!(filter.unscheduled, Some(false));
        assert_eq!(filter.parent, ParentScope::Any);
    }
}
