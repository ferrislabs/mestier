use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{DisplayFields, ReplaceDisplayFieldsCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::SettingsPath, response::SupportSettingsResponse, ticket::require_manager};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReplaceSupportSettingsRequest {
    #[schema(example = json!(["team", "plan"]))]
    pub display_fields: Vec<String>,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/support/settings",
    operation_id = "replaceSupportSettings",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = ReplaceSupportSettingsRequest,
    responses(
        (status = 200, description = "The stored ordered list of custom-field keys", body = inline(DataEnvelope<SupportSettingsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "More than 8 keys, a duplicate, or a key that is empty or longer than 64 characters"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SettingsPath { organization_id }: SettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ReplaceSupportSettingsRequest>,
) -> Result<Response<SupportSettingsResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;

    let display_fields = DisplayFields::try_from(payload.display_fields).map_err(|error| {
        match ApiError::from(error) {
            ApiError::Conflict(message) => ApiError::UnprocessableEntity(message),
            other => other,
        }
    })?;

    let stored = state
        .usecase
        .replace_support_display_fields(ReplaceDisplayFieldsCommand {
            actor,
            organization_id,
            display_fields,
        })
        .await?;

    Ok(Response::OK(stored.into()))
}
