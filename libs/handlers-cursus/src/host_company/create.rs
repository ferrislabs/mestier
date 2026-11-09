use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateHostCompanyCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::HostCompanyCollectionPath, require_org_membership, response::HostCompanyResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateHostCompanyRequest {
    pub name: String,
    pub siret: Option<String>,
    pub address: Option<String>,
    pub legal_form: Option<String>,
    pub naf_code: Option<String>,
    pub employee_count: Option<i32>,
    pub director_name: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/host-companies",
    operation_id = "createCursusHostCompany",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateHostCompanyRequest,
    responses(
        (status = 201, description = "The host company was created", body = inline(DataEnvelope<HostCompanyResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Invalid host company"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: HostCompanyCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateHostCompanyRequest>,
) -> Result<Response<HostCompanyResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let created = state
        .usecase
        .acting_as(user_id)
        .create_host_company(CreateHostCompanyCommand {
            actor,
            organization_id: path.organization_id,
            name: payload.name,
            siret: payload.siret,
            address: payload.address,
            legal_form: payload.legal_form,
            naf_code: payload.naf_code,
            employee_count: payload.employee_count,
            director_name: payload.director_name,
        })
        .await?;

    Ok(Response::Created(created.into()))
}
