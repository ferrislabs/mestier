//! The planning read model: `GET /planning` and `GET /planning/availability`.
//!
//! Read-only by construction — no create/update/delete lives here. See the
//! planning module design doc's invariant 1: the API never refuses an
//! assignment for unavailability, it only reports it. The typed paths this
//! aggregate answers on (`PlanningPath`, `PlanningAvailabilityPath`) live in
//! the crate-level `paths.rs`, shared with whichever future aggregate needs
//! the same organization-scoped root.

use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState};

pub mod get_availability;
pub mod get_planning;

/// This aggregate's routes, unlayered — the crate-level `router` in
/// `lib.rs` merges every aggregate submodule's router before applying the
/// shared rate-limit/auth middleware once, rather than each submodule
/// layering its own.
pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(get_planning::handler)
        .typed_get(get_availability::handler)
}

/// The API's reading window is capped at 92 days — invariant 6 in the
/// planning module design doc, mirrored by `work-time`'s own `GET` (W3).
/// Both `GET /planning` (a calendar-day `DateRange`) and `GET
/// /planning/availability` (an exact `TimeRange` instant span) enforce it:
/// an unbounded availability check is a trivial resource-exhaustion
/// trigger (`detect_conflicts` walks every day of the window, and the
/// repository loads every entry in it), and two endpoints of the same
/// module treating the same notion of "too wide a window" differently is
/// the kind of inconsistency a contributor can't guess and will reproduce.
///
/// The two windows have different shapes (a calendar-day range vs. an
/// exact instant span), so each handler reduces its own to a day count in
/// its own way — but the cap, the status and the message live here once.
pub(crate) const MAX_WINDOW_DAYS: i64 = 92;

/// The `400` both handlers return when their window exceeds
/// `MAX_WINDOW_DAYS`.
pub(crate) fn window_too_wide_error() -> ApiError {
    ApiError::BadRequest(format!(
        "planning window must not exceed {MAX_WINDOW_DAYS} days"
    ))
}
