use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::OrganizationId;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::FieldDayEndPath, resolve_field_actor, response::DayLogResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct EndDayRequest {
    /// When the working day ended. Chosen by the employee, who is often
    /// declaring an earlier moment than now, and defaults to now when absent.
    pub ended_at: Option<DateTime<Utc>>,
}

/// Declares the caller's working day over, closing whatever is still running.
///
/// The calendar day is derived from the organization's timezone, inside the
/// use case, so this route never has to know it.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/field/day-end",
    operation_id = "endWorkingDay",
    tag = crate::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    request_body = EndDayRequest,
    responses(
        (status = 201, description = "Day closed", body = inline(DataEnvelope<DayLogResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Caller is not an employee of this organization"),
        (status = 409, description = "The declared time predates the running entry"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldDayEndPath { organization_id }: FieldDayEndPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(body): Json<EndDayRequest>,
) -> Result<Response<DayLogResponse>, ApiError> {
    let actor = resolve_field_actor(&state, &identity, organization_id).await?;

    let day_log = state
        .usecase
        .end_day(
            organization_id,
            actor.employee_id,
            body.ended_at.unwrap_or_else(Utc::now),
        )
        .await?;

    Ok(Response::Created(day_log.into()))
}
