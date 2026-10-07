use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateSupplierCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use super::{paths::SuppliersPath, response::SupplierResponse};
use crate::require_org_membership;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSupplierRequest {
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
    post,
    path = "/api/v1/organizations/{organization_id}/suppliers",
    operation_id = "createSupplier",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateSupplierRequest,
    responses(
        (status = 201, description = "Supplier created", body = inline(DataEnvelope<SupplierResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Supplier conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: SuppliersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateSupplierRequest>,
) -> Result<Response<SupplierResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let supplier = state
        .usecase
        .acting_as(user_id)
        .create_supplier(CreateSupplierCommand {
            actor,
            organization_id: path.organization_id,
            name: payload.name,
            registration_number: payload.registration_number,
            vat_number: payload.vat_number,
            address: payload.address,
            phone: payload.phone,
            email: payload.email,
        })
        .await?;

    Ok(Response::Created(supplier.into()))
}
