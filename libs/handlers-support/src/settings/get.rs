use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::GetDisplayFieldsCommand;

use crate::{paths::SettingsPath, require_view_support, response::SupportSettingsResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/support/settings",
    operation_id = "getSupportSettings",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The ordered custom-field keys shown on cards and as table columns; empty until configured", body = inline(DataEnvelope<SupportSettingsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SettingsPath { organization_id }: SettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SupportSettingsResponse>, ApiError> {
    require_view_support(&state, &identity, organization_id).await?;

    let display_fields = state
        .usecase
        .get_support_display_fields(GetDisplayFieldsCommand { organization_id })
        .await?;

    Ok(Response::OK(display_fields.into()))
}
