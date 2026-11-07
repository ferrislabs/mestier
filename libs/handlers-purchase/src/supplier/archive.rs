use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{OrganizationId, SupplierId};

use super::{
    load_in_organization,
    paths::{SupplierArchivePath, SupplierRestorePath},
    response::SupplierResponse,
};
use crate::require_org_membership;

async fn set_archived(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    supplier_id: SupplierId,
    archived: bool,
) -> Result<Response<SupplierResponse>, ApiError> {
    require_org_membership(state, identity, organization_id).await?;
    load_in_organization(state, organization_id, supplier_id).await?;
    let (user_id, actor) = resolve_actor(state, identity).await?;

    let supplier = state
        .usecase
        .acting_as(user_id)
        .archive_supplier(supplier_id, archived, actor)
        .await?;

    Ok(Response::OK(supplier.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/archive",
    operation_id = "archiveSupplier",
    tag = crate::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("supplier_id" = SupplierId, Path, description = "Supplier identifier"),
    ),
    responses(
        (status = 200, description = "Supplier archived", body = inline(DataEnvelope<SupplierResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Supplier not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive_handler(
    SupplierArchivePath {
        organization_id,
        supplier_id,
    }: SupplierArchivePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SupplierResponse>, ApiError> {
    set_archived(&state, &identity, organization_id, supplier_id, true).await
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/restore",
    operation_id = "restoreSupplier",
    tag = crate::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("supplier_id" = SupplierId, Path, description = "Supplier identifier"),
    ),
    responses(
        (status = 200, description = "Supplier restored", body = inline(DataEnvelope<SupplierResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Supplier not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn restore_handler(
    SupplierRestorePath {
        organization_id,
        supplier_id,
    }: SupplierRestorePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SupplierResponse>, ApiError> {
    set_archived(&state, &identity, organization_id, supplier_id, false).await
}
