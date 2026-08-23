use axum_extra::routing::TypedPath;
use mestier_core::OrganizationId;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations")]
pub struct OrganizationsPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}")]
pub struct OrganizationPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/legal-identity")]
pub struct OrganizationLegalIdentityPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/users/@me/organizations")]
pub struct CurrentUserOrganizationsPath;
