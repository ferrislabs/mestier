use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
    FeatureFlag, OrganizationId, SetSelfServiceFeatureCommand, application::policy,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    paths::{OrganizationFeaturePath, OrganizationFeaturesPath},
    require_org_membership, resolve_identity_user,
};

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct SelfServiceFeatureResponse {
    pub key: String,
    pub label: String,
    pub description: String,
    pub enabled: bool,
}

impl From<(FeatureFlag, bool)> for SelfServiceFeatureResponse {
    fn from((flag, enabled): (FeatureFlag, bool)) -> Self {
        Self {
            key: flag.key().to_owned(),
            label: flag.label().to_owned(),
            description: flag.description().to_owned(),
            enabled,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct OrganizationFeaturesResponse {
    pub features: Vec<String>,
    pub self_service: Vec<SelfServiceFeatureResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetSelfServiceFeatureRequest {
    pub enabled: bool,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct SelfServiceFeatureToggleResponse {
    pub key: String,
    pub enabled: bool,
    pub changed: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/features",
    operation_id = "getOrganizationFeatures",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Keys of the feature flags enabled for the organization, and the self-service features with their label, description and state", body = inline(DataEnvelope<OrganizationFeaturesResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    OrganizationFeaturesPath { organization_id }: OrganizationFeaturesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<OrganizationFeaturesResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let features = state.usecase.enabled_features(organization_id).await?;

    Ok(Response::OK(OrganizationFeaturesResponse {
        self_service: FeatureFlag::self_service_states(&features)
            .into_iter()
            .map(Into::into)
            .collect(),
        features: features
            .into_iter()
            .map(|flag| flag.key().to_owned())
            .collect(),
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/features/{flag}",
    operation_id = "setOrganizationSelfServiceFeature",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("flag" = String, Path, description = "Self-service feature key"),
    ),
    request_body = SetSelfServiceFeatureRequest,
    responses(
        (status = 200, description = "The feature state after the call; `changed` is false when nothing had to change", body = inline(DataEnvelope<SelfServiceFeatureToggleResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden, requires MANAGE_ORG"),
        (status = 404, description = "Organization not found, not a member of it, or the key is not a self-service feature"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn put_handler(
    OrganizationFeaturePath {
        organization_id,
        flag,
    }: OrganizationFeaturePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SetSelfServiceFeatureRequest>,
) -> Result<Response<SelfServiceFeatureToggleResponse>, ApiError> {
    let user = resolve_identity_user(&state, &identity).await?;
    let flag = FeatureFlag::try_from(flag.as_str()).map_err(|_| ApiError::NotFound)?;
    let iam_roles = match &identity {
        Identity::User(u) => u.roles.clone(),
        Identity::Client(c) => c.roles.clone(),
    };

    let changed = state
        .usecase
        .set_self_service_feature(SetSelfServiceFeatureCommand {
            actor: policy::user_subject(user.id, iam_roles),
            user_id: user.id,
            organization_id,
            flag,
            enabled: payload.enabled,
        })
        .await?;

    Ok(Response::OK(SelfServiceFeatureToggleResponse {
        key: flag.key().to_owned(),
        enabled: payload.enabled,
        changed,
    }))
}
