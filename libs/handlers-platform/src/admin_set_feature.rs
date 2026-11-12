use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    AdminOrganizationFeaturePath, feature_flag, platform_admin,
    response::AdminFeatureToggleResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetFeatureRequest {
    pub enabled: bool,
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/organizations/{organization_id}/features/{flag}",
    operation_id = "setOrganizationFeature",
    tag = crate::ADMIN_TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("flag" = String, Path, description = "Feature flag key"),
    ),
    request_body = SetFeatureRequest,
    responses(
        (status = 200, description = "The flag state after the call; `changed` is false when nothing had to change", body = inline(DataEnvelope<AdminFeatureToggleResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found"),
        (status = 422, description = "Unknown feature flag"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AdminOrganizationFeaturePath {
        organization_id,
        flag,
    }: AdminOrganizationFeaturePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SetFeatureRequest>,
) -> Result<Response<AdminFeatureToggleResponse>, ApiError> {
    let admin = platform_admin(&identity)?;
    let flag = feature_flag(&flag)?;
    let (user_id, _) = resolve_actor(&state, &identity).await?;

    let changed = state
        .usecase
        .set_organization_feature(admin, user_id, organization_id, flag, payload.enabled)
        .await?;

    Ok(Response::OK(AdminFeatureToggleResponse {
        key: flag.key().to_owned(),
        enabled: payload.enabled,
        changed,
    }))
}
