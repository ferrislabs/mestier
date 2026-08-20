use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::OrganizationId;

use crate::{paths::FieldCurrentPath, resolve_field_actor, response::FieldCurrentResponse};

/// What the caller is clocked on to right now, plus whether they already
/// declared the day over.
///
/// The field app calls this on every load: `running` decides whether the
/// screen offers "start" or "stop", and `day_ended_at` decides whether it
/// offers to end the day again or shows it already closed — read from the
/// server rather than from whatever the last mutation happened to answer, so
/// a reload never forgets it.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/field/current",
    operation_id = "getCurrentTimeEntry",
    tag = crate::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "The running entry, if any, and whether the day is already over", body = inline(DataEnvelope<FieldCurrentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Caller is not an employee of this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldCurrentPath { organization_id }: FieldCurrentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<FieldCurrentResponse>, ApiError> {
    let actor = resolve_field_actor(&state, &identity, organization_id).await?;

    let (running, day_log) = state
        .usecase
        .find_field_status(organization_id, actor.employee_id)
        .await?;

    Ok(Response::OK(FieldCurrentResponse {
        running: running.map(Into::into),
        day_ended_at: day_log.map(|log| log.ended_at),
    }))
}
