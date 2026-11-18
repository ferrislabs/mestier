use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::daf::{DafPath, require_daf, response::DafResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/dafs/{daf_id}",
    operation_id = "getDaf",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("daf_id" = mestier_core::DafId, Path, description = "DAF identifier"),
    ),
    responses(
        (status = 200, description = "The DAF", body = inline(DataEnvelope<DafResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DAF not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DafPath {
        organization_id,
        daf_id,
    }: DafPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DafResponse>, ApiError> {
    let daf = require_daf(&state, &identity, organization_id, daf_id).await?;

    Ok(Response::OK(daf.into()))
}
