use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::CustomUnitsPath, require_org_membership, response::CustomUnitResponse};

/// Not paginated: an organization's own units are a handful of rows picked
/// from a dropdown, not a collection that grows with the organization.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/custom-units",
    operation_id = "listCustomUnits",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The organization's custom units", body = inline(DataEnvelope<Vec<CustomUnitResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CustomUnitsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<CustomUnitResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let units = state
        .usecase
        .list_custom_units(path.organization_id)
        .await?;
    let items: Vec<CustomUnitResponse> = units.into_iter().map(CustomUnitResponse::from).collect();

    Ok(Response::OK(items))
}
