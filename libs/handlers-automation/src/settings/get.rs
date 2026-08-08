use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::SettingsPath, require_org_membership, response::AutomationSettingsResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/settings",
    operation_id = "getAutomationSettings",
    tag = super::super::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Current settings, or the defaults when none were ever saved", body = inline(DataEnvelope<AutomationSettingsResponse>)),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: SettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AutomationSettingsResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let settings = state
        .usecase
        .get_automation_settings(path.organization_id)
        .await?;

    Ok(Response::OK(settings.into()))
}
