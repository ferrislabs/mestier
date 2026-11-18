use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{DafDesignation, DafNumber, UpdateDafCommand};

use crate::daf::{DafPath, request::UpdateDafRequest, require_daf, response::DafResponse};

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/dafs/{daf_id}",
    operation_id = "updateDaf",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("daf_id" = mestier_core::DafId, Path, description = "DAF identifier"),
    ),
    request_body = UpdateDafRequest,
    responses(
        (status = 200, description = "The DAF, replaced", body = inline(DataEnvelope<DafResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DAF not found"),
        (status = 409, description = "A number outside 1 to 999 or already taken, a blank designation, more than 20 attachments, or a DAF modified since `expected_updated_at`"),
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
    Json(payload): Json<UpdateDafRequest>,
) -> Result<Response<DafResponse>, ApiError> {
    require_daf(&state, &identity, organization_id, daf_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let daf = state
        .usecase
        .update_daf(UpdateDafCommand {
            actor,
            id: daf_id,
            number: DafNumber::try_from(payload.number)?,
            designation: DafDesignation::try_from(payload.designation)?,
            price_ref: payload.price_ref,
            supplier: payload.supplier,
            installer: payload.installer,
            specs: payload.specs,
            observations: payload.observations,
            photo_key: payload.photo_key,
            attachments: payload.attachments,
            project_owner: payload.project_owner,
            project_manager: payload.project_manager,
            status: payload.status,
            sent_on: payload.sent_on,
            issued_on: payload.issued_on,
            returned_on: payload.returned_on,
            review_note: payload.review_note,
            expected_updated_at: payload.expected_updated_at,
        })
        .await?;

    Ok(Response::OK(daf.into()))
}
