use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::ValidateInterventionCommand;

use crate::{
    maintenance_intervention::{InterventionResponse, ValidateInterventionPath},
    require_org_membership,
};

/// Validates a proposed visit. It now moves the line's due date.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/validate",
    operation_id = "validateMaintenanceIntervention",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("intervention_id" = mestier_core::InterventionId, Path, description = "Intervention identifier"),
    ),
    responses(
        (status = 200, description = "The visit, validated", body = inline(DataEnvelope<InterventionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "No such intervention in this organization"),
        (status = 409, description = "Already validated or rejected"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ValidateInterventionPath {
        organization_id,
        intervention_id,
    }: ValidateInterventionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<InterventionResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let intervention = state
        .usecase
        .validate_maintenance_intervention(
            organization_id,
            ValidateInterventionCommand {
                actor,
                id: intervention_id,
            },
        )
        .await?;

    Ok(Response::OK(intervention.into()))
}
