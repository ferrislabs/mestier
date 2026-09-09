use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateCustomUnitCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::CustomUnitsPath, require_org_membership, response::CustomUnitResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCustomUnitRequest {
    pub code: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/custom-units",
    operation_id = "createCustomUnit",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateCustomUnitRequest,
    responses(
        (status = 201, description = "Custom unit created", body = inline(DataEnvelope<CustomUnitResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Custom unit conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CustomUnitsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateCustomUnitRequest>,
) -> Result<Response<CustomUnitResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let custom_unit = state
        .usecase
        .acting_as(user_id)
        .create_custom_unit(CreateCustomUnitCommand {
            actor,
            organization_id: path.organization_id,
            code: payload.code,
        })
        .await?;

    Ok(Response::Created(custom_unit.into()))
}
