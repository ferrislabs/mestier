use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;

use crate::{
    AbsenceId, AbsenceKind, EmployeeId, MemberId, MinuteInterval, Task, TaskId, TaskLabel,
    TaskStatus,
};

pub mod ports;
pub mod service;

/// The organization's IANA timezone, backed by `chrono_tz::Tz` directly —
/// real DST-aware arithmetic. `work_time` (W3) briefly reserved the name
/// `mestier_core::Tz` for an inert `String` wrapper and dropped it once
/// `expand_work_slots` turned out not to need it (everything it consumes is
/// already local calendar data). W4 is the workstream that actually needs
/// zoned conversions — reconciling `TIMESTAMPTZ` absences/work orders
/// against local-minute work-time intervals — so the name is claimed here
/// instead (see the planning module design doc).
pub type Tz = chrono_tz::Tz;

/// One row of the planning grid: a member of the organization.
///
/// This used to be two variants — an `Employee` row and a `Member` row —
/// because someone was only plannable once they had an HR record, and the read
/// model had to reconcile the two rosters by `user_id`. A member is now a seat
/// that exists on its own, so there is one kind of row and one identifier.
///
/// The contract is optional and stays flattened here rather than nested: a
/// member with no HR profile carries `None`, which is exactly "no rate is set",
/// distinct from `Some(0)` — "genuinely free" (see the planning module design
/// doc's invariant 4).
#[derive(Debug, Clone, PartialEq)]
pub struct PlanningResource {
    pub member_id: MemberId,
    pub display_name: String,
    /// The contractual profile attached to this member, when there is one.
    pub employee_id: Option<EmployeeId>,
    pub hourly_rate_cents: Option<i32>,
    /// `0` when the member has no contract — nothing is planned against a
    /// weekly base they do not have.
    pub weekly_contract_minutes: i32,
}

impl PlanningResource {
    /// The front's canonical key, `member:<uuid>` — part of the API contract
    /// (drag & drop, indexing). Unconditional now: there is a single kind of
    /// row, so there is nothing left to discriminate.
    pub fn resource_id(&self) -> String {
        format!("member:{}", self.member_id)
    }
}

/// A block on the planning grid. A union discriminated by kind — the seam
/// W8 (external sources) extends with a third variant, touching neither the
/// existing tables nor the front already written for these two (see the
/// planning module design doc).
#[derive(Debug, Clone, PartialEq)]
pub enum PlanningEntry {
    Task {
        id: TaskId,
        parent_task_id: Option<TaskId>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        all_day: bool,
        status: TaskStatus,
        blocks_availability: bool,
        /// The number of direct children — always `0` for a subtask, since
        /// the hierarchy is capped at two levels.
        child_count: i64,
        title: String,
        customer_name: Option<String>,
        context_label: Option<String>,
        description: Option<String>,
        member_ids: Vec<MemberId>,
        /// Every label currently attached to this task, built from
        /// `task_label_links` — see `MestierUseCase::get_planning`
        /// (`application/planning/mod.rs`), which batch-loads these after
        /// `PlanningService::get_planning` returns rather than inside it,
        /// for the same reason `patch_task` composes `TaskLabelRepository`
        /// at the application seam: `task`/`planning` and `task_label` stay
        /// separate aggregates.
        labels: Vec<TaskLabel>,
    },
    Absence {
        id: AbsenceId,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        all_day: bool,
        absence_kind: AbsenceKind,
        note: Option<String>,
        member_id: MemberId,
    },
}

/// A task projected for the planning read model: the domain `Task` (with
/// its `assignments`) plus the display fields it does not itself carry —
/// `customer_name`/`context_label` (both `None` for a task with no
/// customer) and `child_count`, computed alongside it in one grouped query
/// rather than loaded from the hierarchy (see the planning module design
/// doc's N+1 warning). Feeds both `PlanningEntry::Task` (via the extra
/// fields) and, via `.task`, `detect_conflicts`'s `busy` argument.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanningTask {
    pub task: Task,
    pub customer_name: Option<String>,
    pub context_label: Option<String>,
    pub child_count: i64,
}

/// One member's work time for the read window: every day
/// `expand_work_slots` produced an entry for, keyed by calendar date. A day
/// absent from `days` means "not worked" — never an empty `Vec` (mirrors
/// `expand_work_slots`'s own sparse-map contract).
///
/// Keyed on the member, like the grid row it feeds: someone with no contract
/// still has work slots, and their availability still has to be readable.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberWorkTime {
    pub member_id: MemberId,
    pub days: BTreeMap<NaiveDate, Vec<MinuteInterval>>,
}

/// The complete `GET /planning` read model.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanningView {
    pub timezone: String,
    pub resources: Vec<PlanningResource>,
    pub entries: Vec<PlanningEntry>,
    pub work_time: Vec<MemberWorkTime>,
}

/// A UTC instant window — the unit `detect_conflicts` and the availability
/// check operate on. Absences and work orders are `TIMESTAMPTZ`, so this is
/// the natural unit to compare them against, unlike work-time's
/// local-calendar `DateRange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeRange {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

impl TimeRange {
    pub fn new(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Result<Self, CoreError> {
        if ends_at <= starts_at {
            return Err(CoreError::Conflict(
                "planning time range ends_at must be after starts_at".to_owned(),
            ));
        }

        Ok(Self { starts_at, ends_at })
    }
}

/// Why a resource is unavailable for a checked window. The API never
/// refuses an assignment for this reason (invariant 1 in the planning
/// module design doc) — it only reports it, and the caller decides.
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictKind {
    Absence {
        kind: AbsenceKind,
        note: Option<String>,
    },
    OutsideWorkHours,
    OverlappingTask {
        task_id: TaskId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    pub kind: ConflictKind,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

/// One resource's availability for a single checked window.
#[derive(Debug, Clone, PartialEq)]
pub struct AvailabilityReport {
    pub resource: PlanningResource,
    pub conflicts: Vec<Conflict>,
}

impl AvailabilityReport {
    pub fn available(&self) -> bool {
        self.conflicts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn time_range_rejects_ends_at_not_after_starts_at() {
        let starts_at = now();

        let err = TimeRange::new(starts_at, starts_at).unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn time_range_accepts_ends_at_after_starts_at() {
        let starts_at = now();
        let ends_at = starts_at + chrono::Duration::hours(1);

        let range = TimeRange::new(starts_at, ends_at).unwrap();

        assert_eq!(range.starts_at, starts_at);
        assert_eq!(range.ends_at, ends_at);
    }

    fn resource(member_id: MemberId) -> PlanningResource {
        PlanningResource {
            member_id,
            display_name: "Alice".to_owned(),
            employee_id: None,
            hourly_rate_cents: None,
            weekly_contract_minutes: 0,
        }
    }

    #[test]
    fn resource_id_keys_on_the_member() {
        let member_id = MemberId(Uuid::new_v4());

        assert_eq!(
            resource(member_id).resource_id(),
            format!("member:{member_id}")
        );
    }

    /// A member with a contract keys exactly like one without: the profile is
    /// carried, never used as the identifier. That is what let the front drop
    /// its `employee:`/`member:` branching.
    #[test]
    fn resource_id_ignores_the_attached_profile() {
        let member_id = MemberId(Uuid::new_v4());
        let with_profile = PlanningResource {
            employee_id: Some(EmployeeId(Uuid::new_v4())),
            hourly_rate_cents: Some(3500),
            weekly_contract_minutes: 2100,
            ..resource(member_id)
        };

        assert_eq!(
            with_profile.resource_id(),
            resource(member_id).resource_id()
        );
    }

    #[test]
    fn availability_report_is_available_without_conflicts() {
        let report = AvailabilityReport {
            resource: resource(MemberId(Uuid::new_v4())),
            conflicts: Vec::new(),
        };

        assert!(report.available());
    }

    #[test]
    fn availability_report_is_unavailable_with_a_conflict() {
        let report = AvailabilityReport {
            resource: resource(MemberId(Uuid::new_v4())),
            conflicts: vec![Conflict {
                kind: ConflictKind::OutsideWorkHours,
                starts_at: now(),
                ends_at: now() + chrono::Duration::hours(1),
            }],
        };

        assert!(!report.available());
    }
}
