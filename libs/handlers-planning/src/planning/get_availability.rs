use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
    AbsenceKind, AvailabilityReport, Conflict, ConflictKind, TimeRange, WorkOrderId,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{paths::PlanningAvailabilityPath, require_org_membership};

/// Mirrors the `starts_at`/`ends_at`/`all_day` triple work orders and
/// absences are created with, so the front can check availability with the
/// exact same values it is about to submit. `starts_at`/`ends_at` are
/// already the precise `TIMESTAMPTZ` bounds either way — `all_day` does not
/// change the computed window, it is accepted for contract parity (see the
/// planning module design doc's decision on all-day entries).
#[derive(Debug, Deserialize)]
pub struct AvailabilityQuery {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    #[serde(default)]
    #[allow(dead_code)]
    pub all_day: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConflictResponse {
    Absence {
        reason: AbsenceKind,
        note: Option<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    },
    OutsideWorkHours {
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    },
    OverlappingWorkOrder {
        work_order_id: WorkOrderId,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    },
}

impl From<Conflict> for ConflictResponse {
    fn from(value: Conflict) -> Self {
        match value.kind {
            ConflictKind::Absence { kind, note } => Self::Absence {
                reason: kind,
                note,
                starts_at: value.starts_at,
                ends_at: value.ends_at,
            },
            ConflictKind::OutsideWorkHours => Self::OutsideWorkHours {
                starts_at: value.starts_at,
                ends_at: value.ends_at,
            },
            ConflictKind::OverlappingWorkOrder { work_order_id } => Self::OverlappingWorkOrder {
                work_order_id,
                starts_at: value.starts_at,
                ends_at: value.ends_at,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AvailabilityResourceResponse {
    pub resource_id: String,
    pub available: bool,
    pub conflicts: Vec<ConflictResponse>,
}

impl From<AvailabilityReport> for AvailabilityResourceResponse {
    fn from(value: AvailabilityReport) -> Self {
        Self {
            resource_id: value.resource.resource_id(),
            available: value.available(),
            conflicts: value
                .conflicts
                .into_iter()
                .map(ConflictResponse::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AvailabilityResponse {
    pub resources: Vec<AvailabilityResourceResponse>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/planning/availability",
    operation_id = "getPlanningAvailability",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("starts_at" = chrono::DateTime<chrono::Utc>, Query, description = "Candidate window start"),
        ("ends_at" = chrono::DateTime<chrono::Utc>, Query, description = "Candidate window end"),
        ("all_day" = Option<bool>, Query, description = "Whether the candidate entry is all-day"),
    ),
    responses(
        (status = 200, description = "Every resource of the organization, annotated with its conflicts for the window", body = inline(DataEnvelope<AvailabilityResponse>)),
        (status = 400, description = "`ends_at` not after `starts_at`, or window wider than 92 days"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PlanningAvailabilityPath { organization_id }: PlanningAvailabilityPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<AvailabilityQuery>,
) -> Result<Response<AvailabilityResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let window = TimeRange::new(query.starts_at, query.ends_at)?;
    validate_window_width(window)?;

    let reports = state
        .usecase
        .get_availability(organization_id, window)
        .await?;

    Ok(Response::OK(AvailabilityResponse {
        resources: reports
            .into_iter()
            .map(AvailabilityResourceResponse::from)
            .collect(),
    }))
}

/// Rejects `window` if it spans more than `MAX_WINDOW_DAYS` — the same cap
/// `/planning` applies to its own calendar-day range, applied here to an
/// exact instant span so a caller cannot request, say, ten years of
/// availability in one call. A duration comparison rather than a
/// `.num_days()` count: the latter truncates towards zero, which would let
/// a window a few hours past the 92-day mark slip through.
fn validate_window_width(window: TimeRange) -> Result<(), ApiError> {
    if window.ends_at - window.starts_at > chrono::Duration::days(super::MAX_WINDOW_DAYS) {
        return Err(super::window_too_wide_error());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::FromRequestParts;
    use mestier_core::{EmployeeId, PlanningResource};

    // `uuid` is not a direct dependency of `handlers-planning` (a
    // `Cargo.toml` this workstream does not own), so fixture ids are parsed
    // from literal strings via `FromStr` rather than generated.
    fn employee_id() -> EmployeeId {
        "11111111-1111-1111-1111-111111111111".parse().unwrap()
    }

    fn work_order_id() -> WorkOrderId {
        "22222222-2222-2222-2222-222222222222".parse().unwrap()
    }

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn a_window_of_exactly_92_days_is_accepted() {
        let starts_at = now();
        let window = TimeRange::new(starts_at, starts_at + chrono::Duration::days(92)).unwrap();

        assert!(validate_window_width(window).is_ok());
    }

    #[test]
    fn a_window_wider_than_92_days_is_a_bad_request() {
        let starts_at = now();
        let window = TimeRange::new(starts_at, starts_at + chrono::Duration::days(93)).unwrap();

        let err = validate_window_width(window).unwrap_err();

        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[test]
    fn a_window_a_few_hours_past_92_days_is_still_rejected() {
        // Guards the duration-comparison choice over `.num_days()`, which
        // truncates towards zero and would let this slip through as "92".
        let starts_at = now();
        let window = TimeRange::new(
            starts_at,
            starts_at + chrono::Duration::days(92) + chrono::Duration::hours(1),
        )
        .unwrap();

        let err = validate_window_width(window).unwrap_err();

        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[test]
    fn absence_conflict_serializes_reason_and_note_alongside_the_absence_tag() {
        let response: ConflictResponse = Conflict {
            kind: ConflictKind::Absence {
                kind: AbsenceKind::Sick,
                note: Some("Grippe".to_owned()),
            },
            starts_at: now(),
            ends_at: now(),
        }
        .into();

        let value = serde_json::to_value(&response).unwrap();

        assert_eq!(value["kind"], "absence");
        assert_eq!(value["reason"], "SICK");
        assert_eq!(value["note"], "Grippe");
    }

    #[test]
    fn outside_work_hours_conflict_serializes_with_its_own_tag() {
        let response: ConflictResponse = Conflict {
            kind: ConflictKind::OutsideWorkHours,
            starts_at: now(),
            ends_at: now(),
        }
        .into();

        let value = serde_json::to_value(&response).unwrap();

        assert_eq!(value["kind"], "outside_work_hours");
        assert!(value.get("reason").is_none());
    }

    #[test]
    fn overlapping_work_order_conflict_serializes_the_work_order_id() {
        let response: ConflictResponse = Conflict {
            kind: ConflictKind::OverlappingWorkOrder {
                work_order_id: work_order_id(),
            },
            starts_at: now(),
            ends_at: now(),
        }
        .into();

        let value = serde_json::to_value(&response).unwrap();

        assert_eq!(value["kind"], "overlapping_work_order");
        assert_eq!(value["work_order_id"], work_order_id().to_string());
    }

    #[test]
    fn available_resource_has_no_conflicts_and_a_true_flag() {
        let report = AvailabilityReport {
            resource: PlanningResource::Employee {
                employee_id: employee_id(),
                user_id: None,
                display_name: "Alice".to_owned(),
                hourly_rate_cents: None,
                weekly_contract_minutes: 0,
            },
            conflicts: Vec::new(),
        };

        let response = AvailabilityResourceResponse::from(report);

        assert_eq!(response.resource_id, format!("employee:{}", employee_id()));
        assert!(response.available);
        assert!(response.conflicts.is_empty());
    }

    /// `NaiveDate` is the only type used as a `Query` field elsewhere in
    /// this crate (`work_time`'s `from`/`to`) -- `DateTime<Utc>` has no
    /// precedent here, so this proves axum's `Query` extractor actually
    /// parses an RFC 3339 timestamp out of a real query string rather than
    /// assuming it from the type deriving `Deserialize`.
    ///
    /// Driven with a manual single poll rather than `#[tokio::test]`:
    /// `tokio` is not a dependency of this crate (a `Cargo.toml` this
    /// workstream does not own), and `Query`'s extraction never actually
    /// awaits anything, so one poll is enough.
    #[test]
    fn availability_query_parses_starts_at_ends_at_and_all_day_from_a_query_string() {
        let uri: axum::http::Uri =
            "http://example.com/x?starts_at=2026-08-10T08:00:00Z&ends_at=2026-08-10T09:00:00Z&all_day=true"
                .parse()
                .unwrap();
        let request = axum::http::Request::builder().uri(uri).body(()).unwrap();
        let (mut parts, ()) = request.into_parts();

        let result = poll_once(Query::<AvailabilityQuery>::from_request_parts(
            &mut parts,
            &(),
        ));
        let Query(query) = result.expect("the query string must parse into AvailabilityQuery");

        assert_eq!(query.starts_at, utc(2026, 8, 10, 8, 0));
        assert_eq!(query.ends_at, utc(2026, 8, 10, 9, 0));
        assert!(query.all_day);
    }

    /// Polls a future once with a no-op waker. Valid here because `Query`'s
    /// `FromRequestParts` implementation resolves synchronously — it never
    /// awaits I/O, it just parses the already-buffered query string.
    fn poll_once<F: std::future::Future>(future: F) -> F::Output {
        use std::{
            pin::pin,
            task::{Context, Poll, Waker},
        };

        let mut future = pin!(future);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("expected the query extractor to resolve synchronously"),
        }
    }

    fn utc(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
        use chrono::TimeZone;
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }
}
