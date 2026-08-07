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
use handlers::AppState;

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
