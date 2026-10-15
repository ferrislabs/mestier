use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::Grid;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{maintenance_overview::MaintenanceGridPath, require_org_membership};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GridQuery {
    /// The season's year.
    pub season: i32,
}

/// The season sheet: every active contract with its lines, the visits of each
/// week and each line's due date as of today.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/maintenance/grid",
    operation_id = "getMaintenanceGrid",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        GridQuery,
    ),
    responses(
        (status = 200, description = "The season grid", body = inline(DataEnvelope<Grid>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MaintenanceGridPath { organization_id }: MaintenanceGridPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<GridQuery>,
) -> Result<Response<Grid>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let grid = state
        .usecase
        .maintenance_grid(actor, organization_id, query.season)
        .await?;

    Ok(Response::OK(grid))
}
