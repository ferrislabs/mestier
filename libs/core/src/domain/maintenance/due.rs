//! When a line must next be done. Pure: `today` is a parameter, never the
//! clock, so week and year boundaries are tested without one.

use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

use crate::domain::maintenance::{Cadence, Season};

/// Ordered by priority: the first that applies wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DueStatus {
    /// An open task (backlog, planned or in progress) already covers it.
    Planned,
    /// Due before the Monday of the current week.
    Overdue,
    ThisWeek,
    NextWeek,
    Later,
    /// Season over, every window done, or on demand.
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct Due {
    /// `None` when `status` is `Done`.
    pub on: Option<NaiveDate>,
    pub status: DueStatus,
}

/// What `due_for` needs to know about one line.
#[derive(Debug, Clone, Copy)]
pub struct LineHistory<'a> {
    /// Dates of the line's validated interventions, any order.
    pub validated_on: &'a [NaiveDate],
    /// Whether a non-cancelled, not-done task carries this line.
    pub has_open_task: bool,
}

/// Rules, in `docs/chantiers/entretien.md`, section "Calcul de l'échéance":
/// - `Rolling`: last validated + `every_days`; without one, the season's
///   first day.
/// - `Windows`: Monday of the first window holding no validated
///   intervention; overdue once that window has ended.
/// - `OnDemand`: always `Done`.
/// - Nothing is due after `season.last_day()`.
pub fn due_for(
    season: Season,
    cadence: &Cadence,
    history: LineHistory<'_>,
    today: NaiveDate,
) -> Due {
    let _ = (season, cadence, history, today);
    todo!("W1: implement and test spec-ent-1 to spec-ent-4 and spec-ent-14")
}
