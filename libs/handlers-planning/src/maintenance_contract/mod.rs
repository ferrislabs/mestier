//! Maintenance contracts: a customer's site kept up for one season, with the
//! lines of work done there. The due dates and the interventions are the
//! interventions workstream's routes. See `docs/chantiers/entretien.md`.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState, resolve_actor};
use mestier_core::{MaintenanceContract, MaintenanceContractId, OrganizationId};
use serde::Deserialize;

use crate::require_org_membership;

pub mod archive;
pub mod create;
pub mod duplicate;
pub mod get_one;
pub mod list;
pub mod request;
pub mod response;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_put(update::handler)
        .typed_post(archive::handler)
        .typed_post(duplicate::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/maintenance/contracts")]
pub struct MaintenanceContractsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}")]
pub struct MaintenanceContractPath {
    pub organization_id: OrganizationId,
    pub contract_id: MaintenanceContractId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}/archive")]
pub struct MaintenanceContractArchivePath {
    pub organization_id: OrganizationId,
    pub contract_id: MaintenanceContractId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}/duplicate"
)]
pub struct MaintenanceContractDuplicatePath {
    pub organization_id: OrganizationId,
    pub contract_id: MaintenanceContractId,
}

/// Checks the caller belongs to `organization_id` and holds `planning.manage`
/// there, then loads the contract in that organization. A contract of another
/// organization is a 404, never a 403.
pub(crate) async fn require_contract(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    contract_id: MaintenanceContractId,
) -> Result<MaintenanceContract, ApiError> {
    require_org_membership(state, identity, organization_id).await?;
    let (_, actor) = resolve_actor(state, identity).await?;

    Ok(state
        .usecase
        .get_maintenance_contract(actor, organization_id, contract_id)
        .await?)
}
