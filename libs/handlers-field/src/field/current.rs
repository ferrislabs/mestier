use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::OrganizationId;

use crate::{paths::FieldCurrentPath, resolve_field_actor, response::TimeEntryResponse};

/// What the caller is clocked on to right now, or `null`.
///
/// The field app calls this on every load: it is what decides whether the
/// screen offers "start" or "stop".
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/field/current",
    operation_id = "getCurrentTimeEntry",
    tag = crate::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "The running entry, or null", body = inline(DataEnvelope<Option<TimeEntryResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Caller is not an employee of this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldCurrentPath { organization_id }: FieldCurrentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Option<TimeEntryResponse>>, ApiError> {
    let actor = resolve_field_actor(&state, &identity, organization_id).await?;

    let running = state
        .usecase
        .find_running_time_entry(actor.employee_id)
        .await?;

    Ok(Response::OK(running.map(Into::into)))
}
