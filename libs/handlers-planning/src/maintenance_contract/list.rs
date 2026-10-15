use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    maintenance_contract::{MaintenanceContractsPath, response::MaintenanceContractResponse},
    require_org_membership,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListMaintenanceContractsQuery {
    /// The season's year.
    pub season: i32,
    /// Archived contracts are left out unless asked for.
    #[serde(default)]
    pub include_archived: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/maintenance/contracts",
    operation_id = "listMaintenanceContracts",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListMaintenanceContractsQuery,
    ),
    responses(
        (status = 200, description = "The season's contracts with their lines, ordered by reference", body = inline(DataEnvelope<Vec<MaintenanceContractResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: MaintenanceContractsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListMaintenanceContractsQuery>,
) -> Result<Response<Vec<MaintenanceContractResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let contracts = state
        .usecase
        .list_maintenance_contracts(
            actor,
            path.organization_id,
            query.season,
            query.include_archived,
        )
        .await?;

    Ok(Response::OK(
        contracts.into_iter().map(Into::into).collect(),
    ))
}
