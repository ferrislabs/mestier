//! The weekly maintenance recap as a PDF, for the Thursday planning meeting.
//! See docs/chantiers/entretien.md, section "PDF du récap".

use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use auth::Identity;
use axum::{
    Extension,
    body::Body,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{NaiveDate, Weekday};
use doc_render::{MaintenanceRecap, TypstRecapRenderer};
use handlers::{ApiError, AppState, resolve_actor};
use http::{
    StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use mestier_core::OrganizationId;
use serde::Deserialize;
use tokio::sync::Semaphore;

use crate::require_org_membership;

pub fn router(_state: &AppState) -> axum::Router<AppState> {
    axum::Router::new().typed_get(handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/maintenance/recap.pdf")]
pub struct RecapPdfPath {
    pub organization_id: OrganizationId,
}

#[derive(Debug, Deserialize)]
pub struct RecapPdfQuery {
    /// ISO week, `2026-W43`.
    pub week: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/maintenance/recap.pdf",
    operation_id = "exportMaintenanceRecapPdf",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("week" = String, Query, description = "ISO week of the recap, e.g. 2026-W43"),
    ),
    responses(
        (status = 200, description = "The weekly maintenance recap as a PDF, generated on demand", content_type = "application/pdf"),
        (status = 400, description = "The week is not an ISO week such as 2026-W43"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    RecapPdfPath { organization_id }: RecapPdfPath,
    Query(query): Query<RecapPdfQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    require_planning_manage(&state, &identity, organization_id).await?;
    let week_start = parse_week(&query.week)?;

    let recap = recap_document(&state, organization_id, week_start).await?;
    let bytes = render_off_the_executor(recap).await?;

    Ok((
        StatusCode::OK,
        [
            (CONTENT_TYPE, "application/pdf".to_owned()),
            (CONTENT_DISPOSITION, content_disposition(week_start)),
        ],
        Body::from(bytes),
    )
        .into_response())
}

/// The data seam. WS3 builds the recap (`GET /maintenance/recap`); this must
/// call the same use case and map its lines to `RecapRow`. Until it lands the
/// PDF is laid out from `MaintenanceRecap::sample`, so the route, the
/// permission and the layout can be exercised.
async fn recap_document(
    _state: &AppState,
    _organization_id: OrganizationId,
    week_start: NaiveDate,
) -> Result<MaintenanceRecap, ApiError> {
    Ok(MaintenanceRecap::sample(week_start))
}

async fn require_planning_manage(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<(), ApiError> {
    let (user_id, _) = resolve_actor(state, identity).await?;
    let iam_roles = match identity {
        Identity::User(user) => user.roles.clone(),
        Identity::Client(client) => client.roles.clone(),
    };
    state
        .usecase
        .authorize_action(user_id, iam_roles, organization_id, "planning.manage")
        .await?;
    Ok(())
}

/// `2026-W43` to the Monday of that week.
fn parse_week(raw: &str) -> Result<NaiveDate, ApiError> {
    let invalid =
        || ApiError::BadRequest(format!("week `{raw}` is not an ISO week such as 2026-W43"));
    let (year, week) = raw.split_once("-W").ok_or_else(invalid)?;
    if year.len() != 4 || !(1..=2).contains(&week.len()) {
        return Err(invalid());
    }
    let year: i32 = year.parse().map_err(|_| invalid())?;
    let week: u32 = week.parse().map_err(|_| invalid())?;
    NaiveDate::from_isoywd_opt(year, week, Weekday::Mon).ok_or_else(invalid)
}

fn content_disposition(week_start: NaiveDate) -> String {
    let week = week_start.format("%G-S%V");
    format!("attachment; filename=\"recap-entretien-{week}.pdf\"")
}

/// Recaps rendering at the same time, across the whole process. The recap is
/// small next to a DOE, so its own pool keeps a Thursday-morning rush of
/// recaps from queueing behind DOE exports, and the other way round.
const CONCURRENT_RENDERS: usize = 2;

const RENDER_TIMEOUT: Duration = Duration::from_secs(30);

static RENDER_PERMITS: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(CONCURRENT_RENDERS)));

async fn render_off_the_executor(recap: MaintenanceRecap) -> Result<Vec<u8>, ApiError> {
    let permit = RENDER_PERMITS
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| ApiError::Internal)?;
    let task = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        TypstRecapRenderer.render(&recap)
    });
    match tokio::time::timeout(RENDER_TIMEOUT, task).await {
        Ok(Ok(Ok(bytes))) => Ok(bytes),
        Ok(Ok(Err(error))) => {
            tracing::error!(error = %error, "maintenance recap rendering failed");
            Err(ApiError::Internal)
        }
        Ok(Err(error)) => {
            tracing::error!(error = %error, "maintenance recap rendering task failed");
            Err(ApiError::Internal)
        }
        Err(_) => {
            tracing::warn!("maintenance recap rendering exceeded {RENDER_TIMEOUT:?}");
            Err(ApiError::Internal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn a_week_parses_to_its_monday() {
        assert_eq!(parse_week("2026-W43").unwrap(), date(2026, 10, 19));
        assert_eq!(parse_week("2026-W1").unwrap(), date(2025, 12, 29));
        assert_eq!(parse_week("2026-W53").unwrap(), date(2026, 12, 28));
    }

    #[test]
    fn anything_else_is_a_bad_request() {
        for raw in [
            "",
            "2026",
            "2026-43",
            "2026-W",
            "2026-W00",
            "2026-W54",
            "2027-W53",
            "26-W43",
            "2026-W043",
            "2026-w43",
            "x-W43",
            "2026-W4x",
        ] {
            assert!(
                matches!(parse_week(raw), Err(ApiError::BadRequest(_))),
                "{raw:?} should be refused"
            );
        }
    }

    #[test]
    fn the_filename_is_ascii_and_names_the_iso_week() {
        let header = content_disposition(date(2026, 10, 19));

        assert_eq!(
            header,
            "attachment; filename=\"recap-entretien-2026-S43.pdf\""
        );
        assert!(http::HeaderValue::from_str(&header).is_ok());
    }

    #[tokio::test]
    async fn the_recap_renders_off_the_executor() {
        let recap = MaintenanceRecap::sample(date(2026, 10, 19));

        let bytes = render_off_the_executor(recap).await.unwrap();

        assert!(bytes.starts_with(b"%PDF"));
    }
}
