use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{OrganizationId, SupplierId};

use super::{load_in_organization, paths::SupplierPath, response::SupplierResponse};
use crate::require_org_membership;

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}",
    operation_id = "getSupplier",
    tag = crate::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("supplier_id" = SupplierId, Path, description = "Supplier identifier"),
    ),
    responses(
        (status = 200, description = "Supplier details", body = inline(DataEnvelope<SupplierResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Supplier not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SupplierPath {
        organization_id,
        supplier_id,
    }: SupplierPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SupplierResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let supplier = load_in_organization(&state, organization_id, supplier_id).await?;

    Ok(Response::OK(supplier.into()))
}
