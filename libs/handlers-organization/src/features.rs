use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::OrganizationId;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{paths::OrganizationFeaturesPath, require_org_membership};

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct OrganizationFeaturesResponse {
    pub features: Vec<String>,
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
        (status = 200, description = "Keys of the feature flags enabled for the organization", body = inline(DataEnvelope<OrganizationFeaturesResponse>)),
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
        features: features
            .into_iter()
            .map(|flag| flag.key().to_owned())
            .collect(),
    }))
}
