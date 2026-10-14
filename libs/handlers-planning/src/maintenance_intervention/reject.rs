use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::RejectInterventionCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    maintenance_intervention::{InterventionResponse, RejectInterventionPath},
    require_org_membership,
};

#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct RejectInterventionRequest {
    /// Why, for the person who proposed it. Not blank when present.
    #[serde(default)]
    pub note: Option<String>,
}

/// Rejects a proposed visit. The line's due date stays where it was.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/reject",
    operation_id = "rejectMaintenanceIntervention",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("intervention_id" = mestier_core::InterventionId, Path, description = "Intervention identifier"),
    ),
    request_body = RejectInterventionRequest,
    responses(
        (status = 200, description = "The visit, rejected", body = inline(DataEnvelope<InterventionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "No such intervention in this organization"),
        (status = 409, description = "Already validated or rejected, or a blank note"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    RejectInterventionPath {
        organization_id,
        intervention_id,
    }: RejectInterventionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    payload: Option<Json<RejectInterventionRequest>>,
) -> Result<Response<InterventionResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let note = payload.and_then(|Json(body)| body.note);

    let intervention = state
        .usecase
        .reject_maintenance_intervention(
            organization_id,
            RejectInterventionCommand {
                actor,
                id: intervention_id,
                note,
            },
        )
        .await?;

    Ok(Response::OK(intervention.into()))
}
