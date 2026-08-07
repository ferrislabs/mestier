use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::StopTimeEntryCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::time_entry::{StopTimeEntryPath, TimeEntryResponse, require_time_entry};

#[derive(Debug, Deserialize, ToSchema)]
pub struct StopTimeEntryRequest {
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
    /// Opaque keys previously returned by `POST /api/v1/files`.
    #[serde(default)]
    pub photos_after: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/time-entries/{time_entry_id}/stop",
    operation_id = "stopTimeEntry",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("time_entry_id" = mestier_core::TimeEntryId, Path, description = "Time entry identifier"),
    ),
    request_body = StopTimeEntryRequest,
    responses(
        (status = 200, description = "Time entry stopped", body = inline(DataEnvelope<TimeEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Time entry already stopped"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: StopTimeEntryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<StopTimeEntryRequest>,
) -> Result<Response<TimeEntryResponse>, ApiError> {
    require_time_entry(
        &state,
        &identity,
        path.organization_id,
        path.time_entry_id,
    )
    .await?;

    let time_entry = state
        .usecase
        .stop_time_entry(StopTimeEntryCommand {
            id: path.time_entry_id,
            ended_at: payload.ended_at,
            photos_after: payload.photos_after,
        })
        .await?;

    Ok(Response::OK(time_entry.into()))
}
