use axum_extra::routing::TypedPath;
use mestier_core::OrganizationId;
use serde::Deserialize;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/endpoints")]
pub struct EndpointsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/endpoints/{endpoint_id}")]
pub struct EndpointPath {
    pub organization_id: OrganizationId,
    pub endpoint_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/endpoints/{endpoint_id}/secret")]
pub struct EndpointSecretPath {
    pub organization_id: OrganizationId,
    pub endpoint_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/settings")]
pub struct SettingsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/deliveries")]
pub struct DeliveriesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/deliveries/{delivery_id}/replay")]
pub struct DeliveryReplayPath {
    pub organization_id: OrganizationId,
    pub delivery_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/events")]
pub struct CataloguePath {
    pub organization_id: OrganizationId,
}
