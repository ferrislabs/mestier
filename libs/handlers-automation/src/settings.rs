//! Settings: read and update the retention and retry-schedule knobs
//! `mestier_core::AutomationSettings` carries, validated against
//! `SettingsBounds` on write — refused, never silently clamped, and the
//! refusal names which bound was crossed
//! (`AutomationSettings::validate`'s own `CoreError::Conflict` message).

use auth::Identity;
use axum::{Extension, Json, Router, extract::State};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::SettingsPath, require_org_membership, response::AutomationSettingsBody};

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new().typed_get(get).typed_put(put)
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/settings",
    operation_id = "getAutomationSettings",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Retention and retry-schedule settings, defaulted if never configured", body = inline(DataEnvelope<AutomationSettingsBody>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get(
    SettingsPath { organization_id }: SettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AutomationSettingsBody>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let settings = state
        .usecase
        .get_automation_settings(organization_id)
        .await?;

    Ok(Response::OK(AutomationSettingsBody::from(settings)))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/automation/settings",
    operation_id = "updateAutomationSettings",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = AutomationSettingsBody,
    responses(
        (status = 200, description = "Settings updated", body = inline(DataEnvelope<AutomationSettingsBody>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "A value is outside the instance's bounds — the message names which one"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn put(
    SettingsPath { organization_id }: SettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AutomationSettingsBody>,
) -> Result<Response<AutomationSettingsBody>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let updated = state
        .usecase
        .update_automation_settings(organization_id, payload.into())
        .await?;

    Ok(Response::OK(AutomationSettingsBody::from(updated)))
}
