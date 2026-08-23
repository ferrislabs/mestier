//! An employee's cost history: `employee_cost_bases`, versioned by effective
//! date so a raise stops rewriting a report the plan already computed (see
//! #282/#300/#301).
//!
//! Kept self-contained the way `absence` is: its own `router(state)`, its
//! own paths and response, instead of folding into the shared `paths.rs`/
//! `response.rs`/flat `router()` in `lib.rs`. `lib.rs` only gains a
//! `pub mod cost_basis;` and a `.merge(cost_basis::router(state))`.
//!
//! Two of the three routes are bare-id, with no organization in the path:
//! `POST`/`GET .../cost-bases` derive it from the employee row `employee_id`
//! loads, `PATCH /cost-bases/{cost_basis_id}` derives it from the cost basis
//! row itself. Both loads and the authorization that follows happen inside
//! `MestierUseCase`, never in the handler — see
//! `MestierUseCase::set_employee_cost_basis` and
//! `MestierUseCase::correct_employee_cost_basis`. Reading an organization
//! from a path that sits next to a bare id is what turns the id into a
//! cross-tenant IDOR.

use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::AppState;
use mestier_core::{
    EmployeeCostBasis, EmployeeCostBasisId, EmployeeId, OrganizationId, salaried_hourly_rate_cents,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod create;
pub mod list;
pub mod update;

/// This aggregate's routes, unlayered — the crate-level `router` in
/// `lib.rs` merges every aggregate submodule's router before applying the
/// shared rate-limit/auth middleware once, rather than each submodule
/// layering its own.
pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_patch(update::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/employees/{employee_id}/cost-bases")]
pub struct EmployeeCostBasesPath {
    pub employee_id: EmployeeId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/cost-bases/{cost_basis_id}")]
pub struct EmployeeCostBasisPath {
    pub cost_basis_id: EmployeeCostBasisId,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct EmployeeCostBasisResponse {
    pub id: EmployeeCostBasisId,
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub effective_from: NaiveDate,
    /// `null` means this is the currently open version.
    pub effective_to: Option<NaiveDate>,
    pub hourly_rate_cents: Option<i32>,
    pub is_salaried: bool,
    pub monthly_cost_cents: Option<i32>,
    /// What an hour under this version costs, whichever basis it is on.
    /// Sent so a history list can show a comparable figure per row rather
    /// than making the reader convert a monthly amount by eye — see
    /// `EmployeeResponse::effective_hourly_rate_cents`, the same rule.
    pub effective_hourly_rate_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EmployeeCostBasis> for EmployeeCostBasisResponse {
    fn from(value: EmployeeCostBasis) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            employee_id: value.employee_id,
            effective_from: value.effective_from,
            effective_to: value.effective_to,
            hourly_rate_cents: value.hourly_rate_cents,
            is_salaried: value.is_salaried,
            monthly_cost_cents: value.monthly_cost_cents,
            effective_hourly_rate_cents: if value.is_salaried {
                salaried_hourly_rate_cents(value.monthly_cost_cents, value.weekly_contract_minutes)
            } else {
                value.hourly_rate_cents
            },
            weekly_contract_minutes: value.weekly_contract_minutes,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
