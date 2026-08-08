use std::time::Duration;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::AutomationSettings;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::SettingsPath, require_org_membership, response::AutomationSettingsResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSettingsRequest {
    pub event_retention_days: u64,
    pub succeeded_delivery_retention_days: u64,
    /// The number of attempts is the length of this list. There is no separate
    /// attempt count to disagree with it.
    pub retry_schedule_seconds: Vec<u64>,
    pub disable_target_after: Option<u32>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/automation/settings",
    operation_id = "updateAutomationSettings",
    tag = super::super::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    request_body = UpdateSettingsRequest,
    responses(
        (status = 200, description = "Settings saved", body = inline(DataEnvelope<AutomationSettingsResponse>)),
        (status = 409, description = "A value outside the instance bounds; the message names the bound"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: SettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateSettingsRequest>,
) -> Result<Response<AutomationSettingsResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let settings = AutomationSettings {
        event_retention: Duration::from_secs(payload.event_retention_days * 86_400),
        succeeded_delivery_retention: Duration::from_secs(
            payload.succeeded_delivery_retention_days * 86_400,
        ),
        retry_schedule: payload
            .retry_schedule_seconds
            .into_iter()
            .map(Duration::from_secs)
            .collect(),
        disable_target_after: payload.disable_target_after,
    };

    let saved = state
        .usecase
        .acting_as(actor)
        .update_automation_settings(path.organization_id, settings)
        .await?;

    Ok(Response::OK(saved.into()))
}
