use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::RecordInterventionCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    maintenance_intervention::{InterventionResponse, LineInterventionsPath},
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecordInterventionRequest {
    /// Within the contract's season and not in the future.
    pub done_on: NaiveDate,
}

/// A manager's direct entry: the visit is validated at once and moves the
/// line's due date.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/maintenance/lines/{line_id}/interventions",
    operation_id = "recordMaintenanceIntervention",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("line_id" = mestier_core::MaintenanceLineId, Path, description = "Maintenance line identifier"),
    ),
    request_body = RecordInterventionRequest,
    responses(
        (status = 201, description = "The visit, validated", body = inline(DataEnvelope<InterventionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "No such line in this organization"),
        (status = 409, description = "Dated in the future or outside the contract's season"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    LineInterventionsPath {
        organization_id,
        line_id,
    }: LineInterventionsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<RecordInterventionRequest>,
) -> Result<Response<InterventionResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let intervention = state
        .usecase
        .record_maintenance_intervention(
            organization_id,
            RecordInterventionCommand {
                actor,
                line_id,
                done_on: payload.done_on,
            },
        )
        .await?;

    Ok(Response::Created(intervention.into()))
}
