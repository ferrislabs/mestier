use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateHostCompanyCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::HostCompanyPath, request::HostCompanyAddressRequest, require_org_membership,
    response::HostCompanyResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateHostCompanyRequest {
    pub name: String,
    pub group_name: Option<String>,
    pub siret: Option<String>,
    pub legal_form: Option<String>,
    pub naf_code: Option<String>,
    pub capital: Option<String>,
    pub employee_count: Option<i32>,
    pub group_employee_count: Option<i32>,
    pub director_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub activity_domains: Option<String>,
    pub missions: Option<String>,
    pub addresses: Vec<HostCompanyAddressRequest>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/cursus/host-companies/{host_company_id}",
    operation_id = "updateCursusHostCompany",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("host_company_id" = mestier_core::HostCompanyId, Path, description = "Host company identifier"),
    ),
    request_body = UpdateHostCompanyRequest,
    responses(
        (status = 200, description = "The host company was updated", body = inline(DataEnvelope<HostCompanyResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Host company not found in the organization"),
        (status = 409, description = "Invalid host company"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    HostCompanyPath {
        organization_id,
        host_company_id,
    }: HostCompanyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateHostCompanyRequest>,
) -> Result<Response<HostCompanyResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .update_host_company(UpdateHostCompanyCommand {
            actor,
            organization_id,
            id: host_company_id,
            name: payload.name,
            group_name: payload.group_name,
            siret: payload.siret,
            legal_form: payload.legal_form,
            naf_code: payload.naf_code,
            capital: payload.capital,
            employee_count: payload.employee_count,
            group_employee_count: payload.group_employee_count,
            director_name: payload.director_name,
            phone: payload.phone,
            email: payload.email,
            website: payload.website,
            activity_domains: payload.activity_domains,
            missions: payload.missions,
            addresses: payload.addresses.into_iter().map(Into::into).collect(),
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
