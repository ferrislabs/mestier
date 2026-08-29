//! Typed path extractors for the whole crate, kept in one file since every
//! automation route nests under the same `/organizations/{organization_id}/automation`
//! prefix — unlike `handlers-planning`, there is no independent aggregate
//! here that would collide with another workstream editing the same file.

use axum_extra::routing::TypedPath;
use mestier_core::OrganizationId;
use serde::Deserialize;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/connectors")]
pub struct ConnectorsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/events")]
pub struct EventsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/credentials")]
pub struct CredentialsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}")]
pub struct CredentialPath {
    pub organization_id: OrganizationId,
    pub credential_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/rotate"
)]
pub struct CredentialRotatePath {
    pub organization_id: OrganizationId,
    pub credential_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/workflows")]
pub struct WorkflowsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}")]
pub struct WorkflowPath {
    pub organization_id: OrganizationId,
    pub workflow_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/trigger")]
pub struct WorkflowTriggerPath {
    pub organization_id: OrganizationId,
    pub workflow_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions")]
pub struct WorkflowVersionsPath {
    pub organization_id: OrganizationId,
    pub workflow_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/runs")]
pub struct WorkflowRunsPath {
    pub organization_id: OrganizationId,
    pub workflow_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/runs")]
pub struct RunsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/runs/{run_id}")]
pub struct RunPath {
    pub organization_id: OrganizationId,
    pub run_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay")]
pub struct RunReplayPath {
    pub organization_id: OrganizationId,
    pub run_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/automation/settings")]
pub struct SettingsPath {
    pub organization_id: OrganizationId,
}
