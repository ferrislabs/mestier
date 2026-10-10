use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{DoeSupply, DraftSupplyFromProductCommand, ProductId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::doe::{DoeDraftSupplyPath, require_doe};

#[derive(Debug, Deserialize, ToSchema)]
pub struct DraftSupplyRequest {
    pub product_id: ProductId,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/does/{doe_id}/supplies/draft",
    operation_id = "draftDoeSupply",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_id" = mestier_core::DoeId, Path, description = "DOE identifier"),
    ),
    request_body = DraftSupplyRequest,
    responses(
        (status = 200, description = "A supply copied from the product's technical sheet, not saved", body = inline(DataEnvelope<DoeSupply>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE or product not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DoeDraftSupplyPath {
        organization_id,
        doe_id,
    }: DoeDraftSupplyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<DraftSupplyRequest>,
) -> Result<Response<DoeSupply>, ApiError> {
    require_doe(&state, &identity, organization_id, doe_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let supply = state
        .usecase
        .draft_doe_supply(DraftSupplyFromProductCommand {
            actor,
            doe_id,
            product_id: payload.product_id,
        })
        .await?;

    Ok(Response::OK(supply))
}
