//! The two read views built on the due-date computation (the season grid and
//! the weekly recap) and the action that turns recap lines into backlog tasks.
//! See `docs/chantiers/entretien.md`.

use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{IsoWeekNumber, OrganizationId};
use serde::Deserialize;

/// The core types the grid and recap bodies are made of. They are inlined
/// into the path responses, so their own schemas, and every schema they point
/// to, have to be listed in the document by hand; the document lives in a
/// crate that does not depend on the core.
pub mod schemas {
    pub use mestier_core::{
        Cadence, ContractSite, ContractStatus, Due, DueStatus, Grid, GridContract, GridLine,
        GridVisit, IntervalDays, InterventionStatus, IsoWeekNumber, Recap, RecapGroup, RecapLine,
        Season, SiteContact, Window, Windows,
    };
}

pub mod grid;
pub mod plan;
pub mod recap;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(grid::handler)
        .typed_get(recap::handler)
        .typed_post(plan::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/maintenance/grid")]
pub struct MaintenanceGridPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/maintenance/recap")]
pub struct MaintenanceRecapPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/maintenance/recap/plan")]
pub struct MaintenancePlanPath {
    pub organization_id: OrganizationId,
}

/// `2026-W43` into its ISO year and week.
pub(crate) fn parse_iso_week(raw: &str) -> Result<(i32, IsoWeekNumber), ApiError> {
    let invalid = || ApiError::BadRequest(format!("`{raw}` is not an ISO week like 2026-W43"));
    let (year, week) = raw.split_once("-W").ok_or_else(invalid)?;
    let year: i32 = year.parse().map_err(|_| invalid())?;
    let week: u8 = week.parse().map_err(|_| invalid())?;
    let week = IsoWeekNumber::try_from(week).map_err(|_| invalid())?;
    Ok((year, week))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_iso_week_is_year_dash_w_week() {
        let (year, week) = parse_iso_week("2026-W43").unwrap();

        assert_eq!((year, week.get()), (2026, 43));
    }

    #[test]
    fn anything_else_is_a_bad_request() {
        for raw in [
            "2026-43", "2026-W", "W43", "2026-W00", "2026-W54", "x-W1", "",
        ] {
            assert!(
                matches!(parse_iso_week(raw), Err(ApiError::BadRequest(_))),
                "{raw}"
            );
        }
    }
}
