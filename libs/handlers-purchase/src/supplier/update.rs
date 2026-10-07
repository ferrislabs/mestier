use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{OrganizationId, SupplierId, UpdateSupplierCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use super::{load_in_organization, paths::SupplierPath, response::SupplierResponse};
use crate::require_org_membership;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSupplierRequest {
    pub name: String,
    #[serde(default)]
    pub registration_number: Option<String>,
    #[serde(default)]
    pub vat_number: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}",
    operation_id = "updateSupplier",
    tag = crate::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("supplier_id" = SupplierId, Path, description = "Supplier identifier"),
    ),
    request_body = UpdateSupplierRequest,
    responses(
        (status = 200, description = "Supplier updated", body = inline(DataEnvelope<SupplierResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Supplier not found"),
        (status = 409, description = "Supplier conflict"),
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
    Json(payload): Json<UpdateSupplierRequest>,
) -> Result<Response<SupplierResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    load_in_organization(&state, organization_id, supplier_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let supplier = state
        .usecase
        .acting_as(user_id)
        .update_supplier(UpdateSupplierCommand {
            actor,
            id: supplier_id,
            name: payload.name,
            registration_number: payload.registration_number,
            vat_number: payload.vat_number,
            address: payload.address,
            phone: payload.phone,
            email: payload.email,
        })
        .await?;

    Ok(Response::OK(supplier.into()))
}
