use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
    AbsenceKind, DateRange, EmployeeAbsenceId, EmployeeId, MinuteInterval, PlanningEntry,
    PlanningResource, PlanningView, TaskId, TaskStatus, UserId,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{paths::PlanningPath, require_org_membership};

#[derive(Debug, Deserialize)]
pub struct PlanningWindowQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanningResourceKindResponse {
    Employee,
    Member,
}

/// A single flat shape, `kind` telling the front which fields are
/// meaningful — unlike `entries`, `resources` is not a discriminated union
/// with different fields per variant (see the planning module design doc).
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PlanningResourceResponse {
    pub resource_id: String,
    pub kind: PlanningResourceKindResponse,
    pub employee_id: Option<EmployeeId>,
    pub user_id: Option<UserId>,
    pub display_name: String,
    pub hourly_rate_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
}

impl From<PlanningResource> for PlanningResourceResponse {
    fn from(value: PlanningResource) -> Self {
        let resource_id = value.resource_id();
        match value {
            PlanningResource::Employee {
                employee_id,
                user_id,
                display_name,
                hourly_rate_cents,
                weekly_contract_minutes,
            } => Self {
                resource_id,
                kind: PlanningResourceKindResponse::Employee,
                employee_id: Some(employee_id),
                user_id,
                display_name,
                hourly_rate_cents,
                weekly_contract_minutes,
            },
            PlanningResource::Member {
                user_id,
                display_name,
            } => Self {
                resource_id,
                kind: PlanningResourceKindResponse::Member,
                employee_id: None,
                user_id: Some(user_id),
                display_name,
                // A member-only resource has no employee record yet:
                // "not set" and "no contractual base", never `Some(0)`
                // (see the planning module design doc's invariant 4).
                hourly_rate_cents: None,
                weekly_contract_minutes: 0,
            },
        }
    }
}

/// `entries` is a union discriminated by `kind` — the seam W8 (external
/// sources) extends with a third variant, without touching the tables or
/// the front already written for these two.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanningEntryResponse {
    Task {
        id: TaskId,
        parent_task_id: Option<TaskId>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        all_day: bool,
        status: TaskStatus,
        blocks_availability: bool,
        child_count: i64,
        title: String,
        customer_name: Option<String>,
        context_label: Option<String>,
        description: Option<String>,
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

impl From<PlanningEntry> for PlanningEntryResponse {
    fn from(value: PlanningEntry) -> Self {
        match value {
            PlanningEntry::Task {
                id,
                parent_task_id,
                starts_at,
                ends_at,
                all_day,
                status,
                blocks_availability,
                child_count,
                title,
                customer_name,
                context_label,
                description,
                employee_ids,
            } => Self::Task {
                id,
                parent_task_id,
                starts_at,
                ends_at,
                all_day,
                status,
                blocks_availability,
                child_count,
                title,
                customer_name,
                context_label,
                description,
                employee_ids,
            },
            PlanningEntry::Absence {
                id,
                starts_at,
                ends_at,
                all_day,
                absence_kind,
                note,
                employee_id,
            } => Self::Absence {
                id,
                starts_at,
                ends_at,
                all_day,
                absence_kind,
                note,
                employee_id,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, ToSchema)]
pub struct MinuteIntervalResponse {
    pub starts_minute: i16,
    pub ends_minute: i16,
}

impl From<MinuteInterval> for MinuteIntervalResponse {
    fn from(value: MinuteInterval) -> Self {
        Self {
            starts_minute: value.starts_minute,
            ends_minute: value.ends_minute,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PlanningWorkTimeDayResponse {
    pub date: NaiveDate,
    pub intervals: Vec<MinuteIntervalResponse>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PlanningWorkTimeResponse {
    pub employee_id: EmployeeId,
    pub days: Vec<PlanningWorkTimeDayResponse>,
}

impl From<mestier_core::EmployeeWorkTime> for PlanningWorkTimeResponse {
    fn from(value: mestier_core::EmployeeWorkTime) -> Self {
        Self {
            employee_id: value.employee_id,
            days: value
                .days
                .into_iter()
                .map(|(date, intervals)| PlanningWorkTimeDayResponse {
                    date,
                    intervals: intervals
                        .into_iter()
                        .map(MinuteIntervalResponse::from)
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PlanningResponse {
    pub timezone: String,
    pub resources: Vec<PlanningResourceResponse>,
    pub entries: Vec<PlanningEntryResponse>,
    pub work_time: Vec<PlanningWorkTimeResponse>,
}

impl From<PlanningView> for PlanningResponse {
    fn from(value: PlanningView) -> Self {
        Self {
            timezone: value.timezone,
            resources: value
                .resources
                .into_iter()
                .map(PlanningResourceResponse::from)
                .collect(),
            entries: value
                .entries
                .into_iter()
                .map(PlanningEntryResponse::from)
                .collect(),
            work_time: value
                .work_time
                .into_iter()
                .map(PlanningWorkTimeResponse::from)
                .collect(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/planning",
    operation_id = "getPlanning",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("from" = chrono::NaiveDate, Query, description = "Window start (inclusive)"),
        ("to" = chrono::NaiveDate, Query, description = "Window end (inclusive)"),
    ),
    responses(
        (status = 200, description = "Resources, entries and work time over the window", body = inline(DataEnvelope<PlanningResponse>)),
        (status = 400, description = "Window wider than 92 days, or `to` before `from`"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PlanningPath { organization_id }: PlanningPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(window): Query<PlanningWindowQuery>,
) -> Result<Response<PlanningResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let range = DateRange::new(window.from, window.to)?;
    validate_window_width(range)?;

    let view = state.usecase.get_planning(organization_id, range).await?;

    Ok(Response::OK(PlanningResponse::from(view)))
}

/// Rejects `range` if it spans more than `MAX_WINDOW_DAYS` calendar days.
/// Extracted so the boundary is directly testable, rather than only
/// reachable by driving the full handler against a live `AppState`.
fn validate_window_width(range: DateRange) -> Result<(), ApiError> {
    if range.span_days() > super::MAX_WINDOW_DAYS {
        return Err(super::window_too_wide_error());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn a_window_of_exactly_92_days_is_accepted() {
        // Matches `DateRange`'s own boundary test: 2026-01-01 to
        // 2026-04-02 spans exactly 92 calendar days, inclusive.
        let range = DateRange::new(date(2026, 1, 1), date(2026, 4, 2)).unwrap();

        assert!(validate_window_width(range).is_ok());
    }

    #[test]
    fn a_window_wider_than_92_days_is_a_bad_request() {
        let range = DateRange::new(date(2026, 1, 1), date(2026, 4, 3)).unwrap();

        let err = validate_window_width(range).unwrap_err();

        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    // `uuid` is not a direct dependency of `handlers-planning` (a
    // `Cargo.toml` this workstream does not own), so fixture ids are parsed
    // from literal strings via `FromStr` rather than generated.
    fn employee_id() -> EmployeeId {
        "11111111-1111-1111-1111-111111111111".parse().unwrap()
    }

    fn user_id() -> UserId {
        "22222222-2222-2222-2222-222222222222".parse().unwrap()
    }

    #[test]
    fn employee_resource_serializes_with_the_employee_kind_and_resource_id() {
        let response: PlanningResourceResponse = PlanningResource::Employee {
            employee_id: employee_id(),
            user_id: Some(user_id()),
            display_name: "Alice".to_owned(),
            hourly_rate_cents: Some(3500),
            weekly_contract_minutes: 2100,
        }
        .into();

        let value = serde_json::to_value(&response).unwrap();

        assert_eq!(value["kind"], "employee");
        assert_eq!(value["resource_id"], format!("employee:{}", employee_id()));
        assert_eq!(value["employee_id"], employee_id().to_string());
        assert_eq!(value["hourly_rate_cents"], 3500);
    }

    #[test]
    fn member_resource_serializes_with_null_employee_fields() {
        let response: PlanningResourceResponse = PlanningResource::Member {
            user_id: user_id(),
            display_name: "Bob".to_owned(),
        }
        .into();

        let value = serde_json::to_value(&response).unwrap();

        assert_eq!(value["kind"], "member");
        assert_eq!(value["resource_id"], format!("member:{}", user_id()));
        assert!(value["employee_id"].is_null());
        assert!(value["hourly_rate_cents"].is_null());
        assert_eq!(value["weekly_contract_minutes"], 0);
    }

    #[test]
    fn task_entry_serializes_with_the_task_tag() {
        let entry: PlanningEntryResponse = PlanningEntry::Task {
            id: "33333333-3333-3333-3333-333333333333".parse().unwrap(),
            parent_task_id: None,
            starts_at: Utc::now(),
            ends_at: Utc::now(),
            all_day: false,
            status: TaskStatus::Planned,
            blocks_availability: true,
            child_count: 0,
            title: "Toiture".to_owned(),
            customer_name: Some("Alice Dupont".to_owned()),
            context_label: Some("Chantier principal".to_owned()),
            description: None,
            employee_ids: vec![employee_id()],
        }
        .into();

        let value = serde_json::to_value(&entry).unwrap();

        assert_eq!(value["kind"], "task");
        assert_eq!(value["customer_name"], "Alice Dupont");
        assert_eq!(value["status"], "PLANNED");
    }

    #[test]
    fn task_entry_without_a_customer_serializes_null_customer_fields() {
        let entry: PlanningEntryResponse = PlanningEntry::Task {
            id: "33333333-3333-3333-3333-333333333333".parse().unwrap(),
            parent_task_id: None,
            starts_at: Utc::now(),
            ends_at: Utc::now(),
            all_day: false,
            status: TaskStatus::Planned,
            blocks_availability: false,
            child_count: 2,
            title: "Réunion d'équipe".to_owned(),
            customer_name: None,
            context_label: None,
            description: None,
            employee_ids: vec![],
        }
        .into();

        let value = serde_json::to_value(&entry).unwrap();

        assert_eq!(value["kind"], "task");
        assert!(value["customer_name"].is_null());
        assert!(value["context_label"].is_null());
        assert_eq!(value["blocks_availability"], false);
        assert_eq!(value["child_count"], 2);
    }

    #[test]
    fn absence_entry_serializes_with_the_absence_tag() {
        let entry: PlanningEntryResponse = PlanningEntry::Absence {
            id: "44444444-4444-4444-4444-444444444444".parse().unwrap(),
            starts_at: Utc::now(),
            ends_at: Utc::now(),
            all_day: true,
            absence_kind: AbsenceKind::Leave,
            note: Some("Congés".to_owned()),
            employee_id: employee_id(),
        }
        .into();

        let value = serde_json::to_value(&entry).unwrap();

        assert_eq!(value["kind"], "absence");
        assert_eq!(value["absence_kind"], "LEAVE");
        assert_eq!(value["employee_id"], employee_id().to_string());
    }
}
