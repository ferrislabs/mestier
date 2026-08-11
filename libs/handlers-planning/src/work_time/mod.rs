//! Employee working time: a recurring weekly rhythm, versioned, and concrete
//! work slots pinned to real dates that take priority over the rhythm for
//! that date. See the planning module design doc for the full model.
//!
//! Owns its own leaf paths (see the crate-level `paths.rs` docstring) since
//! Every route here is addressed by member: work time concerns the person,
//! not their contract. The rhythm is resolved through the profile inside the
//! use case, so no caller ever has to know whether the member has one.

use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::NaiveDate;
use handlers::AppState;
use mestier_core::{EmployeeId, EmployeeRhythm, MemberId, OrganizationId, RhythmSlot, WorkSlot};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod get_work_time;
pub mod put_rhythm;
pub mod put_work_slots;

/// This aggregate's routes, unlayered — the crate-level `router` in
/// `lib.rs` merges every aggregate submodule's router before applying the
/// shared rate-limit/auth middleware once, rather than each submodule
/// layering its own.
pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(get_work_time::handler)
        .typed_put(put_rhythm::handler)
        .typed_put(put_work_slots::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/members/{member_id}/work-time")]
pub struct WorkTimePath {
    pub member_id: MemberId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/members/{member_id}/rhythm")]
pub struct RhythmPath {
    pub member_id: MemberId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/members/{member_id}/work-slots")]
pub struct WorkSlotsPath {
    pub member_id: MemberId,
}

/// `from`/`to` are both required — mirrors the `GET work-time` and `PUT
/// work-slots` contracts, which never default the window.
#[derive(Debug, Deserialize)]
pub struct WorkTimeWindowQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct RhythmSlotResponse {
    pub weekday: i16,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

impl From<RhythmSlot> for RhythmSlotResponse {
    fn from(value: RhythmSlot) -> Self {
        Self {
            weekday: value.weekday,
            starts_minute: value.starts_minute,
            ends_minute: value.ends_minute,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct RhythmResponse {
    pub id: mestier_core::EmployeeRhythmId,
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub slots: Vec<RhythmSlotResponse>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<EmployeeRhythm> for RhythmResponse {
    fn from(value: EmployeeRhythm) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            employee_id: value.employee_id,
            effective_from: value.effective_from,
            effective_to: value.effective_to,
            slots: value
                .slots
                .into_iter()
                .map(RhythmSlotResponse::from)
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkSlotResponse {
    pub id: mestier_core::WorkSlotId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub work_date: NaiveDate,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

impl From<WorkSlot> for WorkSlotResponse {
    fn from(value: WorkSlot) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            member_id: value.member_id,
            work_date: value.work_date,
            starts_minute: value.starts_minute,
            ends_minute: value.ends_minute,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkTimeResponse {
    pub rhythms: Vec<RhythmResponse>,
    pub work_slots: Vec<WorkSlotResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // `uuid` is not a direct dependency of `handlers-planning` (a
    // `Cargo.toml` this workstream does not own), so fixture ids are parsed
    // from literal strings via `FromStr` rather than generated.
    fn rhythm() -> EmployeeRhythm {
        let now = chrono::Utc::now();
        let id: mestier_core::EmployeeRhythmId =
            "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let organization_id: OrganizationId =
            "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let employee_id: EmployeeId = "33333333-3333-3333-3333-333333333333".parse().unwrap();
        let rhythm_slot_id: mestier_core::RhythmSlotId =
            "44444444-4444-4444-4444-444444444444".parse().unwrap();
        EmployeeRhythm {
            id,
            organization_id,
            employee_id,
            effective_from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            effective_to: None,
            slots: vec![RhythmSlot {
                id: rhythm_slot_id,
                rhythm_id: id,
                weekday: 1,
                starts_minute: 480,
                ends_minute: 720,
            }],
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn rhythm_response_nests_its_slots() {
        let response: RhythmResponse = rhythm().into();

        assert_eq!(response.slots.len(), 1);
        assert_eq!(response.slots[0].weekday, 1);
        assert_eq!(response.slots[0].starts_minute, 480);
    }

    #[test]
    fn work_time_window_query_parses_iso_dates() {
        let query: WorkTimeWindowQuery =
            serde_json::from_value(serde_json::json!({ "from": "2026-08-01", "to": "2026-08-31" }))
                .unwrap();

        assert_eq!(query.from, NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
        assert_eq!(query.to, NaiveDate::from_ymd_opt(2026, 8, 31).unwrap());
    }
}
