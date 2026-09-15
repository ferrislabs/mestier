use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    CustomerContextId, CustomerId, MemberId, OrganizationId, ProjectId, QuoteId, TaskRecurrenceId,
};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TaskId(pub Uuid);

impl FromStr for TaskId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TaskId)
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// How much of a series a `DELETE` on one occurrence removes.
///
/// A single verb (soft-delete) with a scope, not two endpoints: the caller
/// genuinely has to choose, and a scope parameter keeps that choice at one
/// call site instead of two routes that would otherwise duplicate every
/// other check `soft_delete_occurrence` makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteScope {
    /// Only this occurrence — the default, and the only meaningful choice
    /// for a task that never belonged to a series.
    ThisOccurrence,
    /// This occurrence and every later one in the same series, identified by
    /// `occurrence_date`. A task that does not belong to a series falls back
    /// to [`Self::ThisOccurrence`]'s own behavior — there is no "later" to
    /// reach for.
    ThisAndFollowing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TaskAssignmentId(pub Uuid);

impl FromStr for TaskAssignmentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TaskAssignmentId)
    }
}

impl Display for TaskAssignmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    /// Work that is agreed but not scheduled. First in declaration order
    /// because it is the earliest point of the progression, and because the
    /// `task_status` enum in PostgreSQL carries `'BACKLOG'` `BEFORE
    /// 'PLANNED'` — the two orders are kept in step so an `ORDER BY status`
    /// reads the same from either side.
    ///
    /// Orthogonal to the window, deliberately and in both directions: a
    /// backlog task usually has no dates, but a status is never derived from
    /// a window and a window is never derived from a status. Dragging a
    /// backlog task onto a calendar gives it dates and leaves it in
    /// `Backlog` until somebody says otherwise; clearing a planned task's
    /// dates leaves it `Planned`. The alternative — deriving one from the
    /// other — means a date affordance silently rewrites the column the user
    /// is looking at.
    Backlog,
    Planned,
    InProgress,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backlog => "BACKLOG",
            Self::Planned => "PLANNED",
            Self::InProgress => "IN_PROGRESS",
            Self::Done => "DONE",
            Self::Cancelled => "CANCELLED",
        }
    }
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BACKLOG" => Ok(Self::Backlog),
            "PLANNED" => Ok(Self::Planned),
            "IN_PROGRESS" => Ok(Self::InProgress),
            "DONE" => Ok(Self::Done),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(format!("invalid task status `{other}`")),
        }
    }
}

/// A reference to a plannable resource, as carried by the `PATCH` payload's
/// `assignees` list (see the planning module design doc).
///
/// One variant, one identifier. It used to be two — an employee record or an
/// organization member who might not have one — and resolving the second
/// triggered on-the-fly employee-record creation. Every member is now
/// assignable on its own, so there is nothing to provision and nothing to
/// discriminate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssigneeRef(pub MemberId);

#[derive(Debug, Clone, PartialEq)]
pub struct TaskAssignment {
    pub id: TaskAssignmentId,
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub member_id: MemberId,
    pub created_at: DateTime<Utc>,
}

/// A task's manual position inside its board column.
///
/// **A string, compared lexicographically, never a number.** Moving a card
/// between two others generates a rank that sorts strictly between its two
/// neighbours' ranks and writes that one row — no other card in the column is
/// touched. The rejected alternative was an `i32` position renumbered on every
/// drop: it costs a write per card below the insertion point, and two people
/// dropping at the same time interleave their renumberings into an order
/// neither asked for. Here, two concurrent drops produce two independent
/// ranks; the worst case is a tie in the eye of the beholder, not a corrupted
/// column.
///
/// The ordering that matters is byte order. `Ord` here is the derived one on
/// the inner `String`, which compares bytes, and the `board_rank` column is
/// declared `TEXT COLLATE "C"` so PostgreSQL compares the same bytes the same
/// way — see `migrations/20260914000002_add_task_board_rank.up.sql`. The
/// database's own collation is `en_US.utf8`, a multi-level linguistic
/// comparison that is under no obligation to agree with byte order; letting
/// the two sides disagree on a single pair is how cards end up jumping around
/// on reload. The alphabet is lowercase-only for the same reason one layer up:
/// case is where locale collations diverge from byte order most often, so a
/// rank never contains an uppercase letter to begin with.
///
/// ## The scale
///
/// A rank is read as the fractional part of a base-36 number over the digits
/// `0-9a-z`, whose byte order is their digit order: `"i"` is roughly one half,
/// `"9"` roughly one quarter, `"1i"` a sliver above `1/36`. Two rules keep
/// string order and numeric order the same thing, both enforced by
/// `chk_tasks_board_rank_shape`:
///
/// * a rank is never empty — that is what `board_rank IS NULL` means instead;
/// * a rank never ends in `0`, the lowest digit. `"1"` and `"10"` denote the
///   same fraction, so a trailing zero would let two distinct strings name one
///   position. [`BoardRank::between`] never emits one.
///
/// ## Growth, and the rebalancing job this chantier does not write
///
/// Ranks get longer as a column is churned, and that is the accepted cost of
/// never renumbering. A generated rank is **at most one character longer than
/// the neighbour it grew out of**, so `n` drops into the same gap produce a
/// rank of at most `n` characters. In practice it is far shorter: each extra
/// character buys `log2(36) ≈ 5.2` halvings of the remaining gap, so a hundred
/// repeated drops at one position land under thirty characters, and a
/// realistic board — where drops spread across the column — grows
/// logarithmically.
///
/// There is deliberately no rebalancing job. It would become worth writing
/// when a single column's ranks routinely pass a few hundred bytes, which
/// takes tens of thousands of drops into the same gap; it would mean
/// rewriting every row of a column inside one transaction, which is exactly
/// the cost this design exists to avoid, and it would have to be reconciled
/// with clients holding the old ranks. Until a column actually degenerates,
/// the length is a number in a `TEXT` column that nobody reads.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
pub struct BoardRank(pub String);

impl Display for BoardRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The digits a rank is spelled with, in byte order — so their position in
/// this string *is* their numeric value, and comparing two ranks byte by byte
/// compares them digit by digit.
const RANK_ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// Base 36. Wide enough that a halved gap survives five insertions per added
/// character, narrow enough to stay inside `[0-9a-z]` — no uppercase, so no
/// collation can reorder a rank behind our back.
const RANK_BASE: u8 = 36;

impl BoardRank {
    /// A rank that sorts strictly between `before` and `after`.
    ///
    /// The three shapes a board asks for, in one function because they are one
    /// operation — "put this card here" — and splitting them would push the
    /// `None` handling into every caller:
    ///
    /// * `(Some(a), Some(b))` — dropped between two cards;
    /// * `(None, Some(b))` — dropped above the first card;
    /// * `(Some(a), None)` — dropped below the last card;
    /// * `(None, None)` — the first card in an empty column, which lands in
    ///   the middle of the scale so the next drop has room on either side.
    ///
    /// Fallible on purpose, and only for inputs that cannot be honoured:
    /// neighbours in the wrong order or equal (a client holding a stale
    /// board), a neighbour outside the alphabet, an empty neighbour, or an
    /// insertion before an all-zero rank — nothing sorts below `"0"`, which is
    /// why the generator never produces one. Every one of those would
    /// otherwise have to be answered with an invented position, and a wrong
    /// rank is written to the database and read back forever.
    pub fn between(before: Option<&Self>, after: Option<&Self>) -> Result<Self, CoreError> {
        if let (Some(before), Some(after)) = (before, after)
            && before >= after
        {
            return Err(CoreError::Conflict(format!(
                "board rank `{before}` does not sort before `{after}`"
            )));
        }

        let lower = match before {
            Some(rank) => rank.digits()?,
            None => Vec::new(),
        };
        let upper = match after {
            Some(rank) => Some(rank.digits()?),
            None => None,
        };

        let digits = between_digits(&lower, upper.as_deref())?;

        Ok(Self(
            digits
                .into_iter()
                .map(|digit| RANK_ALPHABET[digit as usize] as char)
                .collect(),
        ))
    }

    /// `count` ranks, strictly increasing, evenly spread across the gap
    /// between `before` and `after` — the bulk form of [`Self::between`].
    ///
    /// This exists for one job: materializing the order a column already has
    /// but has never written down. Every task predates `board_rank` (the
    /// migration that added the column backfilled nothing), so the first
    /// drop into any column finds every neighbour unranked and has nothing to
    /// bracket. Ranking the column in one pass, in the order it was already
    /// being displayed in, turns that dead end into a single `O(column)`
    /// write that never has to happen again.
    ///
    /// Repeated [`Self::between`] would also produce an ordered sequence, and
    /// it is the wrong tool: bisecting the same gap `n` times makes each rank
    /// about one character longer than the last, so a column of forty cards
    /// ends with a forty-character rank — the exact growth the doc on
    /// [`BoardRank`] describes as the accepted cost of *repeated drops*, paid
    /// here for nothing. Spacing the whole batch at once instead makes every
    /// rank the same width, and that width is the smallest that fits:
    /// `count + 1` intervals need `ceil(log36(count + 1))` digits, so forty
    /// cards get two characters each and a thousand get three.
    ///
    /// Deterministic: the same gap and the same `count` give the same ranks,
    /// which is what makes an initialization reproducible in a test and
    /// diffable in a migration.
    ///
    /// Fallible for the same reason [`Self::between`] is, plus one more: a
    /// gap can be too narrow to hold `count` distinct positions *at any
    /// width* only when it is empty, but a gap can need more digits than the
    /// scale is worth spending, and [`MAX_RANK_WIDTH`] is where that stops.
    pub fn evenly_spaced_between(
        before: Option<&Self>,
        after: Option<&Self>,
        count: usize,
    ) -> Result<Vec<Self>, CoreError> {
        if count == 0 {
            return Ok(Vec::new());
        }

        let lower = match before {
            Some(rank) => rank.digits()?,
            None => Vec::new(),
        };
        let upper = match after {
            Some(rank) => Some(rank.digits()?),
            None => None,
        };

        // Both bounds are read as fractions, so widening the window is a
        // lossless operation: it multiplies both ends by 36 and leaves the
        // gap's *shape* alone while multiplying the number of positions
        // inside it. That is why this can loop — one more digit is always
        // 36 times more room, until the width is not worth paying for.
        let mut width = lower
            .len()
            .max(upper.as_ref().map_or(0, |digits| digits.len()))
            .max(1);

        loop {
            let low = scale_to_width(&lower, width);
            let high = match &upper {
                Some(digits) => scale_to_width(digits, width),
                // No upper neighbour: the top of the scale, which is the
                // value `1` — `36^width` at this width.
                None => RANK_SCALE_BASE.pow(width as u32),
            };

            if high <= low {
                return Err(CoreError::Conflict(format!(
                    "board rank `{}` does not sort before `{}`",
                    before.map(Self::to_string).unwrap_or_default(),
                    after.map(Self::to_string).unwrap_or_default(),
                )));
            }

            let intervals = count as u128 + 1;
            if high - low >= intervals {
                let step = (high - low) / intervals;
                return (1..=count as u128)
                    .map(|index| spell(low + index * step, width))
                    .collect();
            }

            width += 1;
            if width > MAX_RANK_WIDTH {
                return Err(CoreError::Conflict(format!(
                    "cannot fit {count} board ranks between `{}` and `{}`",
                    before.map(Self::to_string).unwrap_or_default(),
                    after.map(Self::to_string).unwrap_or_default(),
                )));
            }
        }
    }

    /// The rank's digits as their numeric values. Rejects anything outside the
    /// alphabet rather than skipping it: the field is public, so a caller —
    /// or a row written before `chk_tasks_board_rank_shape` existed — can hand
    /// over any string at all, and silently dropping a character would move
    /// the card somewhere nobody asked for.
    fn digits(&self) -> Result<Vec<u8>, CoreError> {
        if self.0.is_empty() {
            return Err(CoreError::Conflict(
                "a board rank cannot be empty — an unranked task carries NULL".to_owned(),
            ));
        }

        self.0
            .bytes()
            .map(|byte| match byte {
                b'0'..=b'9' => Ok(byte - b'0'),
                b'a'..=b'z' => Ok(byte - b'a' + 10),
                _ => Err(CoreError::Conflict(format!(
                    "board rank `{self}` is not spelled with `0-9a-z`"
                ))),
            })
            .collect()
    }
}

/// Digits strictly between `lower` and `upper`, both read as base-36
/// fractions — `lower` absent meaning `0`, `upper` absent meaning `1`.
///
/// Walks the two operands digit by digit while they agree, copying what they
/// share, and then settles at the first digit where they differ:
///
/// * a gap of two or more digits leaves room, so it takes the midpoint and
///   stops — the result is one digit longer than the shared prefix, which is
///   the common case and the reason ranks stay short;
/// * a gap of exactly one digit leaves no room at this position, so it keeps
///   `lower`'s digit — which already puts the result strictly below `upper` —
///   and from there only has to beat `lower`'s remaining suffix, with nothing
///   above it. That is [`above`], and it is where the one extra character
///   comes from in the worst case.
fn between_digits(lower: &[u8], upper: Option<&[u8]>) -> Result<Vec<u8>, CoreError> {
    let Some(upper) = upper else {
        return Ok(above(lower));
    };

    let mut digits = Vec::new();
    for (index, high) in upper.iter().copied().enumerate() {
        let low = lower.get(index).copied().unwrap_or(0);

        if high == low {
            digits.push(low);
            continue;
        }

        // `low > high` cannot happen: `between` refused `before >= after`
        // before calling, and the alphabet's byte order is its digit order, so
        // the string comparison it made is this digit comparison. Reported
        // rather than asserted — a panic in the domain is never the answer.
        if high < low {
            return Err(CoreError::Conflict(
                "board rank neighbours are not in ascending order".to_owned(),
            ));
        }

        if high - low >= 2 {
            digits.push((low + high) / 2);
        } else {
            digits.push(low);
            digits.extend(above(lower.get(index + 1..).unwrap_or_default()));
        }

        return Ok(digits);
    }

    // Every digit of `upper` matched `lower`, which makes `upper` a prefix of
    // `lower` — it therefore does not sort after it. Reachable only for an
    // all-zero `upper` such as `"0"`, since `chk_tasks_board_rank_shape`
    // forbids a trailing zero everywhere else, and nothing can be inserted
    // below `"0"`: it is the bottom of the scale.
    Err(CoreError::Conflict(format!(
        "nothing sorts before board rank `{}`",
        upper
            .iter()
            .map(|digit| RANK_ALPHABET[*digit as usize] as char)
            .collect::<String>()
    )))
}

/// Digits strictly above `lower` and strictly below `1` — the "dropped below
/// the last card" case, and the tail of [`between_digits`]'s tight-gap branch.
///
/// Copies `lower`'s leading top digits (`z`, which has nothing above it to
/// aim at) and then splits what is left of the scale. An exhausted `lower`
/// reads as `0`, whose midpoint with `1` is the middle of the alphabet, so an
/// empty column's first rank lands there rather than at an edge.
fn above(lower: &[u8]) -> Vec<u8> {
    let mut digits = Vec::new();

    for index in 0.. {
        let low = lower.get(index).copied().unwrap_or(0);

        if low + 1 < RANK_BASE {
            digits.push((low + RANK_BASE) / 2);
            break;
        }

        digits.push(low);
    }

    digits
}

/// The base as the arithmetic in [`BoardRank::evenly_spaced_between`] needs
/// it. Same 36 as [`RANK_BASE`], widened once here rather than cast at every
/// use — the intermediate values are whole-scale integers, not digits.
const RANK_SCALE_BASE: u128 = RANK_BASE as u128;

/// How many digits an evenly spaced batch may grow to before it is refused.
///
/// Ten base-36 digits is room for `36^10` — about 3.6 quadrillion — distinct
/// positions in a single gap, which no column reaches by any route. The cap
/// is not really about arithmetic room: it is there so a caller that asks for
/// an impossible batch (a `count` read off a corrupted row count, a gap that
/// is empty in a way the `high <= low` check cannot see) fails loudly instead
/// of allocating a rank nobody can read.
const MAX_RANK_WIDTH: usize = 10;

/// `digits`, read as a fraction, multiplied by `36^width` — i.e. the integer
/// the digits spell once padded with trailing zeros out to `width`.
///
/// Padding with zeros is what makes two ranks of different lengths
/// comparable: `"9"` and `"91"` are `9/36` and `9/36 + 1/1296`, and at width
/// two they are `324` and `325`. Nothing is lost, because `width` is never
/// less than the longest input.
fn scale_to_width(digits: &[u8], width: usize) -> u128 {
    (0..width).fold(0u128, |value, index| {
        value * RANK_SCALE_BASE + digits.get(index).copied().unwrap_or(0) as u128
    })
}

/// The inverse of [`scale_to_width`]: `value` written as `width` base-36
/// digits, with trailing zeros dropped.
///
/// Dropping them is not cosmetic. `chk_tasks_board_rank_shape` forbids a
/// trailing zero because `"1"` and `"10"` denote the same fraction, and two
/// strings naming one position is how a total order stops being one. Dropping
/// them changes no value and cannot empty the string: every value this is
/// called with is at least one step above the bottom of its gap.
fn spell(value: u128, width: usize) -> Result<BoardRank, CoreError> {
    let mut digits = vec![0u8; width];
    let mut rest = value;
    for slot in digits.iter_mut().rev() {
        *slot = (rest % RANK_SCALE_BASE) as u8;
        rest /= RANK_SCALE_BASE;
    }

    while digits.last() == Some(&0) {
        digits.pop();
    }

    if digits.is_empty() {
        // Unreachable by construction, and reported rather than asserted for
        // the same reason `between_digits` reports its own impossible branch:
        // a panic in the domain is never the answer.
        return Err(CoreError::Internal(
            "an evenly spaced board rank came out empty".to_owned(),
        ));
    }

    Ok(BoardRank(
        digits
            .into_iter()
            .map(|digit| RANK_ALPHABET[digit as usize] as char)
            .collect(),
    ))
}

/// The planning module's unit: a meeting, a trip, a training session, or a
/// "chantier" — the special case that carries a customer. See the planning
/// module design doc's vocabulary section: a chantier is not a distinct
/// entity, it is a task with `customer_id`/`customer_context_id` set.
///
/// The schema is recursive (`parent_task_id` can itself point at a task with
/// a parent), but the domain caps the hierarchy at two levels — see
/// [`service::validate_parent_depth`]. `starts_at`/`ends_at` are `None` on a
/// subtask that inherits its parent's window, and on any task — root
/// included — that is not scheduled at all; [`service::resolve_task_window`]
/// resolves the effective window at read time and returns `None` when there
/// is none to resolve.
///
/// A root used to be required to carry its own dates, by
/// `chk_tasks_root_has_dates` and redundantly by
/// [`service::TaskService::create_task`]. Both are gone: an undated task is
/// the normal state of everything sitting in [`TaskStatus::Backlog`], and of
/// anything else somebody unscheduled. What survives is the coherence of the
/// pair — both dates or neither, and `ends_at` after `starts_at` —
/// still enforced by `chk_tasks_dates_both_or_neither` and
/// `chk_tasks_ends_at_after_starts_at`.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub organization_id: OrganizationId,
    pub parent_task_id: Option<TaskId>,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub status: TaskStatus,
    /// Declared at creation, never guessed: only a task that carries this as
    /// `true` produces a conflict in `detect_conflicts` — a reminder does
    /// not block anyone's availability, a meeting does.
    pub blocks_availability: bool,
    /// `customer_id`/`customer_context_id` are both present or both absent —
    /// a task with a customer *is* a chantier, nothing more.
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
    /// The project this task is costed against, when it has one. Unrelated to
    /// `parent_task_id`: a subtask attaches to a project directly, which is
    /// what the two-level hierarchy cap used to make impossible. A task with
    /// `None` costs nobody's project, and still counts towards the person who
    /// was assigned to it.
    pub project_id: Option<ProjectId>,
    /// What the task costs beyond somebody's time: travel, a skip, a parking
    /// fee. Free-form on purpose — a kilometric scale is fairer and only works
    /// if distances actually get entered (see ADR 0002). `0` means no expense,
    /// and carries no label: see [`Self::expenses_label`].
    pub expenses_cents: i32,
    /// Why the money was spent. Present exactly when `expenses_cents` is
    /// non-zero, enforced both here (`TaskService`) and by
    /// `chk_tasks_expenses_label_required`. An amount with no reason cannot be
    /// audited three months later, and clearing the amount clears the reason
    /// with it.
    pub expenses_label: Option<String>,
    /// The complete set of employees currently assigned. Always loaded and
    /// persisted as a whole — see the `PATCH` contract: `assignees` is the
    /// full list, never a delta. A subtask never inherits its parent's
    /// assignees: without one of its own, it is assigned to nobody.
    pub assignments: Vec<TaskAssignment>,
    /// The series this occurrence was materialized from, when it still
    /// follows one. `PATCH`ing any of this task's own fields sets this to
    /// `None` — see `service::TaskService::patch_task`'s detach-on-edit
    /// rule: an edited occurrence stops following the rule and keeps its
    /// own life. Never set by `TaskService::create_task`; only
    /// `TaskRecurrenceService::materialize_range` produces a task carrying
    /// this.
    pub recurrence_id: Option<TaskRecurrenceId>,
    /// Where this task sits inside its board column, when somebody has
    /// dragged it. `None` is the ordinary state — every task predating this
    /// field, and every task created since, carries it until a first drop
    /// assigns one, and there is no backfill. `None` sorts **after** every
    /// ranked task, in SQL (`ORDER BY board_rank ASC NULLS LAST`, PostgreSQL's
    /// default for an ascending sort) and in the domain
    /// ([`service::sort_by_board_rank`]) — the two orderings are written
    /// against each other on purpose, because a disagreement shows up only as
    /// cards jumping position on reload.
    ///
    /// A third axis, orthogonal to the other two exactly as `status` and the
    /// window are orthogonal to each other: writing a rank never writes a
    /// status and never writes a window. Dragging a card up its column moves
    /// it within that column and does nothing else; a card that changes
    /// column changes `status`, and that is a different field in the same
    /// `PATCH`.
    pub board_rank: Option<BoardRank>,
    /// The calendar date this occurrence was materialized for, in the
    /// recurrence's own timezone. Kept even after `recurrence_id` is
    /// cleared by an edit, so the occurrence's own history stays readable —
    /// it answers "which Tuesday was this" long after the task stopped
    /// following the series.
    pub occurrence_date: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = TaskId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn task_id_rejects_invalid_uuid() {
        assert!(TaskId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn task_assignment_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = TaskAssignmentId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn task_assignment_id_rejects_invalid_uuid() {
        assert!(TaskAssignmentId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn task_status_parses_known_values() {
        assert_eq!(
            "BACKLOG".parse::<TaskStatus>().unwrap(),
            TaskStatus::Backlog
        );
        assert_eq!(
            "PLANNED".parse::<TaskStatus>().unwrap(),
            TaskStatus::Planned
        );
        assert_eq!(
            "IN_PROGRESS".parse::<TaskStatus>().unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!("DONE".parse::<TaskStatus>().unwrap(), TaskStatus::Done);
        assert_eq!(
            "CANCELLED".parse::<TaskStatus>().unwrap(),
            TaskStatus::Cancelled
        );
    }

    #[test]
    fn task_status_rejects_unknown_values() {
        assert!("SCHEDULED".parse::<TaskStatus>().is_err());
    }

    #[test]
    fn task_status_round_trips_through_as_str() {
        for status in [
            TaskStatus::Backlog,
            TaskStatus::Planned,
            TaskStatus::InProgress,
            TaskStatus::Done,
            TaskStatus::Cancelled,
        ] {
            assert_eq!(status.as_str().parse::<TaskStatus>().unwrap(), status);
        }
    }
    // -- BoardRank ----------------------------------------------------------
    //
    // The generator is the whole of this workstream's risk, so it gets the
    // disproportionate coverage the two pure functions in `service.rs` get:
    // the three shapes a caller can ask for, the inputs it must refuse, the
    // degenerate case that kills a naive midpoint scheme, and a property test
    // standing in for the thousand examples nobody would write by hand.

    mod board_rank_tests {
        use super::*;
        use proptest::prelude::*;

        /// A rank as it comes out of the database or off the wire.
        fn rank(value: &str) -> BoardRank {
            BoardRank(value.to_owned())
        }

        #[test]
        fn a_rank_between_two_neighbours_sorts_strictly_between_them() {
            let before = rank("a");
            let after = rank("c");

            let generated = BoardRank::between(Some(&before), Some(&after)).unwrap();

            assert!(before < generated, "{before:?} < {generated:?}");
            assert!(generated < after, "{generated:?} < {after:?}");
        }

        /// The first edge case: dropping a card above every other one. There
        /// is no lower neighbour, so the rank is generated between the bottom
        /// of the scale and the current first card.
        #[test]
        fn a_rank_before_the_first_sorts_below_it() {
            let first = rank("i");

            let generated = BoardRank::between(None, Some(&first)).unwrap();

            assert!(generated < first, "{generated:?} < {first:?}");
        }

        /// The second edge case: dropping a card below every other one.
        #[test]
        fn a_rank_after_the_last_sorts_above_it() {
            let last = rank("i");

            let generated = BoardRank::between(Some(&last), None).unwrap();

            assert!(last < generated, "{last:?} < {generated:?}");
        }

        /// The third: the first card ever dropped into an empty column. It
        /// lands in the middle of the scale rather than at either end, so the
        /// next card has room on both sides without lengthening anything.
        #[test]
        fn the_first_rank_in_an_empty_column_lands_in_the_middle() {
            let generated = BoardRank::between(None, None).unwrap();

            assert!(BoardRank("0".to_owned()) < generated);
            assert!(generated < BoardRank("z".to_owned()));
        }

        /// Neighbours the wrong way round mean the client is holding a stale
        /// board. Refused rather than answered with an invented position: any
        /// value this could return would be wrong for one of the two
        /// neighbours, and a wrong rank is written to the database and read
        /// back forever.
        #[test]
        fn neighbours_in_the_wrong_order_are_refused() {
            let result = BoardRank::between(Some(&rank("c")), Some(&rank("a")));

            assert!(matches!(result, Err(CoreError::Conflict(_))));
        }

        #[test]
        fn equal_neighbours_are_refused() {
            let result = BoardRank::between(Some(&rank("a")), Some(&rank("a")));

            assert!(matches!(result, Err(CoreError::Conflict(_))));
        }

        /// `BoardRank` has a public field, so nothing stops a caller — or a
        /// row written before `chk_tasks_board_rank_shape` existed — from
        /// handing over something outside the alphabet. The generator checks
        /// rather than assuming.
        #[test]
        fn a_neighbour_outside_the_alphabet_is_refused() {
            assert!(BoardRank::between(Some(&rank("A")), None).is_err());
            assert!(BoardRank::between(Some(&rank("a b")), None).is_err());
            assert!(BoardRank::between(None, Some(&rank("é"))).is_err());
        }

        #[test]
        fn an_empty_neighbour_is_refused() {
            assert!(BoardRank::between(Some(&rank("")), None).is_err());
            assert!(BoardRank::between(None, Some(&rank(""))).is_err());
        }

        /// Nothing sorts before a rank that is all zeroes, which is the whole
        /// reason the generator never emits a trailing zero and
        /// `chk_tasks_board_rank_shape` refuses one. Asked to do the
        /// impossible, this reports it.
        #[test]
        fn inserting_before_an_all_zero_rank_is_refused() {
            assert!(BoardRank::between(None, Some(&rank("0"))).is_err());
        }

        /// **The degenerate case.** A naive midpoint over a fixed-width string
        /// dies here: after a few dozen insertions into the same gap it runs
        /// out of room and starts emitting a value equal to one of its
        /// neighbours. Every intermediate rank is kept and the whole set is
        /// checked at the end, so a collision anywhere in the hundred fails
        /// the test, not just one at the boundary.
        #[test]
        fn a_hundred_insertions_into_the_same_gap_stay_distinct_and_ordered() {
            let before = rank("a");
            let after = rank("b");

            // Each new card is dropped immediately below `before`, which is
            // the worst case for the scheme: the gap under attack never
            // widens.
            let mut lower = before.clone();
            let mut generated = Vec::new();
            for _ in 0..100 {
                let next = BoardRank::between(Some(&lower), Some(&after)).unwrap();
                assert!(lower < next);
                assert!(next < after);
                generated.push(next.clone());
                lower = next;
            }

            let mut sorted = generated.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), generated.len(), "ranks must stay distinct");
            assert_eq!(sorted, generated, "ranks must come out already ordered");

            // And the same gap attacked from the other side: every new card
            // dropped immediately above `before`.
            let mut upper = after.clone();
            let mut generated = Vec::new();
            for _ in 0..100 {
                let next = BoardRank::between(Some(&before), Some(&upper)).unwrap();
                assert!(before < next);
                assert!(next < upper);
                generated.push(next.clone());
                upper = next;
            }

            let mut sorted = generated.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), generated.len(), "ranks must stay distinct");
            generated.reverse();
            assert_eq!(sorted, generated, "ranks must descend as the gap closes");
        }

        /// The growth bound this scheme accepts, pinned as a test so the
        /// doc comment on [`BoardRank`] cannot quietly stop being true: a
        /// generated rank is never more than one character longer than the
        /// neighbour it grew out of.
        #[test]
        fn a_generated_rank_grows_by_at_most_one_character() {
            let mut lower = rank("a");
            let after = rank("b");

            for _ in 0..100 {
                let next = BoardRank::between(Some(&lower), Some(&after)).unwrap();
                assert!(
                    next.0.len() <= lower.0.len() + 1,
                    "{next:?} is more than one character longer than {lower:?}"
                );
                lower = next;
            }

            // Repeated insertion is linear in the worst case but logarithmic
            // in practice: each extra character buys log2(36) ~ 5 halvings, so
            // a hundred drops into one gap stay well under a hundred
            // characters.
            assert!(
                lower.0.len() < 30,
                "a hundred drops grew to {}",
                lower.0.len()
            );
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(1000))]

            /// A thousand random adjacent pairs, which is the point of a
            /// property test here rather than three examples: the interesting
            /// inputs are the ones nobody thinks to write down — a neighbour
            /// that is a prefix of the other, a gap of exactly one digit, a
            /// rank ending on the top digit of the alphabet.
            #[test]
            fn a_generated_rank_sorts_strictly_between_any_adjacent_pair(
                left in "[0-9a-z]{0,5}[1-9a-z]",
                right in "[0-9a-z]{0,5}[1-9a-z]",
            ) {
                let (before, after) = if left < right {
                    (BoardRank(left), BoardRank(right))
                } else {
                    (BoardRank(right), BoardRank(left))
                };
                prop_assume!(before < after);

                let generated = BoardRank::between(Some(&before), Some(&after)).unwrap();

                prop_assert!(before < generated, "{before:?} < {generated:?}");
                prop_assert!(generated < after, "{generated:?} < {after:?}");
                // The shape `chk_tasks_board_rank_shape` enforces one layer
                // down: never empty, never ending on the lowest digit.
                prop_assert!(!generated.0.is_empty());
                prop_assert!(!generated.0.ends_with('0'));
            }

            /// The two open-ended shapes, held to the same property.
            #[test]
            fn a_generated_rank_sorts_outside_a_single_neighbour(
                value in "[0-9a-z]{0,5}[1-9a-z]",
            ) {
                let neighbour = BoardRank(value);

                let above = BoardRank::between(Some(&neighbour), None).unwrap();
                prop_assert!(neighbour < above, "{neighbour:?} < {above:?}");

                let below = BoardRank::between(None, Some(&neighbour)).unwrap();
                prop_assert!(below < neighbour, "{below:?} < {neighbour:?}");
                prop_assert!(!below.0.ends_with('0'));
            }
        }

        // -- evenly spaced batches ------------------------------------------
        //
        // The bulk generator, which exists to materialize a column that has
        // an order but no ranks. Its risk is different from `between`'s: not
        // "does one rank land in the right place" but "do `n` of them land in
        // the right places, stay short, and keep the shape the column
        // constraint demands".

        /// Every batch this module produces has to hold three properties at
        /// once, and a helper that checks all three keeps each test about the
        /// case it is actually naming.
        fn assert_batch_is_well_formed(
            batch: &[BoardRank],
            before: Option<&BoardRank>,
            after: Option<&BoardRank>,
        ) {
            for rank in batch {
                assert!(!rank.0.is_empty(), "a rank is never empty: {rank:?}");
                assert!(
                    !rank.0.ends_with('0'),
                    "a trailing zero would let two strings name one position: {rank:?}"
                );
                assert!(
                    rank.0
                        .bytes()
                        .all(|b| b.is_ascii_digit() || b.is_ascii_lowercase()),
                    "a rank is spelled with `0-9a-z`: {rank:?}"
                );
                if let Some(before) = before {
                    assert!(before < rank, "{before:?} < {rank:?}");
                }
                if let Some(after) = after {
                    assert!(rank < after, "{rank:?} < {after:?}");
                }
            }

            for pair in batch.windows(2) {
                assert!(pair[0] < pair[1], "{:?} < {:?}", pair[0], pair[1]);
            }
        }

        #[test]
        fn an_empty_column_is_spread_across_the_whole_scale() {
            let batch = BoardRank::evenly_spaced_between(None, None, 5).unwrap();

            assert_eq!(batch.len(), 5);
            assert_batch_is_well_formed(&batch, None, None);
            assert!(
                batch.iter().all(|rank| rank.0.len() == 1),
                "five cards fit in one base-36 digit each: {batch:?}"
            );
        }

        /// The half-ranked column: everything generated must sort above the
        /// cards that already carry a rank, because those keep theirs and the
        /// unranked ones were displayed after them.
        #[test]
        fn a_batch_above_a_lower_bound_stays_above_it() {
            let floor = rank("z");

            let batch = BoardRank::evenly_spaced_between(Some(&floor), None, 6).unwrap();

            assert_eq!(batch.len(), 6);
            assert_batch_is_well_formed(&batch, Some(&floor), None);
        }

        /// The reason this is not a loop around `between`. Bisecting the same
        /// gap `n` times grows a character roughly every five cards; spacing
        /// the batch at once spends the smallest width that fits and spends
        /// it once.
        #[test]
        fn a_batch_is_shorter_than_the_same_count_of_repeated_bisections() {
            let batch = BoardRank::evenly_spaced_between(None, None, 40).unwrap();
            let widest = batch.iter().map(|rank| rank.0.len()).max().unwrap();
            assert_eq!(
                widest, 2,
                "41 intervals need two base-36 digits, and no card needs more: {batch:?}"
            );

            let mut bisected = Vec::new();
            let mut floor = None;
            for _ in 0..40 {
                let next = BoardRank::between(floor.as_ref(), Some(&rank("z"))).unwrap();
                floor = Some(next.clone());
                bisected.push(next);
            }
            let bisected_widest = bisected.iter().map(|rank| rank.0.len()).max().unwrap();
            assert!(
                bisected_widest > widest,
                "repeated bisection is what this generator exists to avoid: {bisected_widest} \
                 vs {widest}"
            );
        }

        /// Reproducible, which is what lets an initialization be asserted
        /// against a literal in a test and read in a diff.
        #[test]
        fn the_same_gap_and_count_always_give_the_same_ranks() {
            let floor = rank("9");
            let ceiling = rank("i");

            let first = BoardRank::evenly_spaced_between(Some(&floor), Some(&ceiling), 7).unwrap();
            let second = BoardRank::evenly_spaced_between(Some(&floor), Some(&ceiling), 7).unwrap();

            assert_eq!(first, second);
            assert_batch_is_well_formed(&first, Some(&floor), Some(&ceiling));
        }

        /// A batch of one is the same question `between` answers, and must
        /// land in the same gap — not necessarily on the same rank, since the
        /// two spend their digits differently.
        #[test]
        fn a_batch_of_one_lands_where_between_would_have() {
            let floor = rank("a");
            let ceiling = rank("c");

            let batch = BoardRank::evenly_spaced_between(Some(&floor), Some(&ceiling), 1).unwrap();

            assert_eq!(batch.len(), 1);
            assert_batch_is_well_formed(&batch, Some(&floor), Some(&ceiling));
        }

        #[test]
        fn a_batch_of_nothing_is_empty_and_reads_nothing() {
            assert_eq!(
                BoardRank::evenly_spaced_between(None, None, 0).unwrap(),
                Vec::new()
            );
        }

        /// A gap that runs the wrong way is a client holding a stale board,
        /// and the batch form refuses it exactly like `between` does.
        #[test]
        fn an_inverted_gap_is_refused() {
            let result = BoardRank::evenly_spaced_between(Some(&rank("c")), Some(&rank("a")), 3);

            assert!(matches!(result, Err(CoreError::Conflict(_))), "{result:?}");
        }

        #[test]
        fn a_gap_with_nothing_below_it_is_refused() {
            // Nothing sorts under `"0"`, the bottom of the scale.
            let result = BoardRank::evenly_spaced_between(None, Some(&rank("0")), 2);

            assert!(matches!(result, Err(CoreError::Conflict(_))), "{result:?}");
        }

        #[test]
        fn a_batch_against_a_neighbour_outside_the_alphabet_is_refused() {
            assert!(BoardRank::evenly_spaced_between(Some(&rank("A")), None, 2).is_err());
            assert!(BoardRank::evenly_spaced_between(Some(&rank("")), None, 2).is_err());
        }

        /// A batch must leave room for the drops that come after it —
        /// initializing a column and then immediately dragging inside it is
        /// the single most likely sequence in production.
        #[test]
        fn a_card_can_still_be_dropped_between_two_freshly_spaced_ranks() {
            let batch = BoardRank::evenly_spaced_between(None, None, 12).unwrap();

            for pair in batch.windows(2) {
                let dropped = BoardRank::between(Some(&pair[0]), Some(&pair[1])).unwrap();
                assert!(pair[0] < dropped && dropped < pair[1], "{dropped:?}");
            }
        }

        proptest! {
            /// The property the three examples above stand for: whatever the
            /// gap and whatever the count, the batch is strictly increasing,
            /// inside the gap, and shaped the way the column constraint
            /// demands.
            #[test]
            fn a_batch_is_ordered_and_inside_its_gap(
                left in "[0-9a-z]{0,4}[1-9a-z]",
                right in "[0-9a-z]{0,4}[1-9a-z]",
                count in 1usize..40,
            ) {
                let (before, after) = if left < right {
                    (BoardRank(left), BoardRank(right))
                } else {
                    (BoardRank(right), BoardRank(left))
                };
                prop_assume!(before < after);

                let batch =
                    BoardRank::evenly_spaced_between(Some(&before), Some(&after), count).unwrap();

                prop_assert_eq!(batch.len(), count);
                for rank in &batch {
                    prop_assert!(before < *rank, "{:?} < {:?}", before, rank);
                    prop_assert!(*rank < after, "{:?} < {:?}", rank, after);
                    prop_assert!(!rank.0.is_empty());
                    prop_assert!(!rank.0.ends_with('0'));
                }
                for pair in batch.windows(2) {
                    prop_assert!(pair[0] < pair[1]);
                }
            }
        }
    }
}
