use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;

use crate::{
    AbsenceKind, EmployeeAbsenceId, EmployeeId, MinuteInterval, UserId, WorkOrder, WorkOrderId,
    WorkOrderStatus,
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

/// One row of the planning grid — never a table, always assembled from
/// `employees` and `organization_members` (see the planning module design
/// doc's dedup rule). `Employee` is the rich side: it carries the hourly
/// rate and the contractual base. `Member` is a drop target with no
/// employee record yet — making the two variants carry different fields
/// (rather than one flat struct with optional employee-only fields) keeps
/// "a member has no rate" unrepresentable as anything but the absence of
/// the field.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanningResource {
    Employee {
        employee_id: EmployeeId,
        user_id: Option<UserId>,
        display_name: String,
        /// `None` means the rate is not set yet; `Some(0)` means genuinely
        /// free — never conflate the two (see the planning module design
        /// doc's invariant 4).
        hourly_rate_cents: Option<i32>,
        weekly_contract_minutes: i32,
    },
    Member {
        user_id: UserId,
        display_name: String,
    },
}

impl PlanningResource {
    /// The front's canonical key, `<kind>:<uuid>` — part of the API
    /// contract (drag & drop, indexing). The employee side keys on
    /// `employee_id`; the member side has none yet, so it keys on
    /// `user_id` — the same identifier `AssigneeRef::Member` carries.
    pub fn resource_id(&self) -> String {
        match self {
            Self::Employee { employee_id, .. } => format!("employee:{employee_id}"),
            Self::Member { user_id, .. } => format!("member:{user_id}"),
        }
    }
}

/// A block on the planning grid. A union discriminated by kind — the seam
/// W8 (external sources) extends with a third variant, touching neither the
/// existing tables nor the front already written for these two (see the
/// planning module design doc).
#[derive(Debug, Clone, PartialEq)]
pub enum PlanningEntry {
    WorkOrder {
        id: WorkOrderId,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        all_day: bool,
        status: WorkOrderStatus,
        title: Option<String>,
        customer_name: String,
        context_label: String,
        note: Option<String>,
        employee_ids: Vec<EmployeeId>,
    },
    Absence {
        id: EmployeeAbsenceId,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        all_day: bool,
        absence_kind: AbsenceKind,
        note: Option<String>,
        employee_id: EmployeeId,
    },
}

/// A work order projected for the planning read model: the domain
/// `WorkOrder` (with its `assignments`) plus the two display fields it does
/// not itself carry, `customer_name` and `context_label`. Feeds both
/// `PlanningEntry::WorkOrder` (via the two extra fields) and, via
/// `.work_order`, `detect_conflicts`'s `busy` argument.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanningWorkOrder {
    pub work_order: WorkOrder,
    pub customer_name: String,
    pub context_label: String,
}

/// One employee's work time for the read window: every day
/// `expand_work_slots` produced an entry for, keyed by calendar date. A day
/// absent from `days` means "not worked" — never an empty `Vec` (mirrors
/// `expand_work_slots`'s own sparse-map contract).
#[derive(Debug, Clone, PartialEq)]
pub struct EmployeeWorkTime {
    pub employee_id: EmployeeId,
    pub days: BTreeMap<NaiveDate, Vec<MinuteInterval>>,
}

/// The complete `GET /planning` read model.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanningView {
    pub timezone: String,
    pub resources: Vec<PlanningResource>,
    pub entries: Vec<PlanningEntry>,
    pub work_time: Vec<EmployeeWorkTime>,
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
    OverlappingWorkOrder {
        work_order_id: WorkOrderId,
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

    #[test]
    fn employee_resource_id_uses_the_employee_id() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let resource = PlanningResource::Employee {
            employee_id,
            user_id: None,
            display_name: "Alice".to_owned(),
            hourly_rate_cents: Some(3500),
            weekly_contract_minutes: 2100,
        };

        assert_eq!(resource.resource_id(), format!("employee:{employee_id}"));
    }

    #[test]
    fn member_resource_id_uses_the_user_id() {
        let user_id = UserId(Uuid::new_v4());
        let resource = PlanningResource::Member {
            user_id,
            display_name: "Bob".to_owned(),
        };

        assert_eq!(resource.resource_id(), format!("member:{user_id}"));
    }

    #[test]
    fn availability_report_is_available_without_conflicts() {
        let report = AvailabilityReport {
            resource: PlanningResource::Member {
                user_id: UserId(Uuid::new_v4()),
                display_name: "Bob".to_owned(),
            },
            conflicts: Vec::new(),
        };

        assert!(report.available());
    }

    #[test]
    fn availability_report_is_unavailable_with_a_conflict() {
        let report = AvailabilityReport {
            resource: PlanningResource::Member {
                user_id: UserId(Uuid::new_v4()),
                display_name: "Bob".to_owned(),
            },
            conflicts: vec![Conflict {
                kind: ConflictKind::OutsideWorkHours,
                starts_at: now(),
                ends_at: now() + chrono::Duration::hours(1),
            }],
        };

        assert!(!report.available());
    }
}
