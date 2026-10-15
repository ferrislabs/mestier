use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::ProposeInterventionCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::FieldInterventionPath, resolve_field_actor, response::FieldInterventionResponse,
};

/// Carries no member: the caller always declares on their own task, resolved
/// from the authenticated identity.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ProposeInterventionRequest {
    /// The day the visit was done: within the contract's season, not in the
    /// future.
    pub done_on: NaiveDate,
}

/// Declares a maintenance visit done on one of the caller's own tasks.
///
/// The visit is only proposed: the line's due date does not move until the
/// manager validates it. Any task that is not assigned to the caller, or is
/// not a maintenance task, is a 404.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/field/tasks/{task_id}/intervention",
    operation_id = "proposeMaintenanceIntervention",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_id" = mestier_core::TaskId, Path, description = "Task identifier"),
    ),
    request_body = ProposeInterventionRequest,
    responses(
        (status = 201, description = "Visit proposed", body = inline(DataEnvelope<FieldInterventionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Caller is not an employee of this organization"),
        (status = 404, description = "No such maintenance task assigned to the caller"),
        (status = 409, description = "Dated in the future or outside the contract's season"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldInterventionPath {
        organization_id,
        task_id,
    }: FieldInterventionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ProposeInterventionRequest>,
) -> Result<Response<FieldInterventionResponse>, ApiError> {
    resolve_field_actor(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let intervention = state
        .usecase
        .propose_maintenance_intervention(ProposeInterventionCommand {
            actor,
            organization_id,
            task_id,
            done_on: payload.done_on,
        })
        .await?;

    Ok(Response::Created(intervention.into()))
}
