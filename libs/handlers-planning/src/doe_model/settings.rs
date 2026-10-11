use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{DoeStyle, UpdateDoeSettingsCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    doe_model::{DoeSettingsPath, response::DoeSettingsResponse},
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDoeSettingsRequest {
    pub style: DoeStyle,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/doe-settings",
    operation_id = "getDoeSettings",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Document settings, or the defaults when none were saved", body = inline(DataEnvelope<DoeSettingsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_handler(
    path: DoeSettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DoeSettingsResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let settings = state.usecase.get_doe_settings(path.organization_id).await?;

    Ok(Response::OK(settings.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/doe-settings",
    operation_id = "updateDoeSettings",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = UpdateDoeSettingsRequest,
    responses(
        (status = 200, description = "Document settings saved", body = inline(DataEnvelope<DoeSettingsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn put_handler(
    path: DoeSettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateDoeSettingsRequest>,
) -> Result<Response<DoeSettingsResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let settings = state
        .usecase
        .acting_as(user_id)
        .update_doe_settings(UpdateDoeSettingsCommand {
            actor,
            organization_id: path.organization_id,
            style: payload.style,
        })
        .await?;

    Ok(Response::OK(settings.into()))
}
