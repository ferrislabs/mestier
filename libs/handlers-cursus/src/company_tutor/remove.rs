use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};
use mestier_core::RemoveCompanyTutorCommand;

use crate::{EmptyResponse, paths::CompanyTutorPath, require_org_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/cursus/company-tutors/{company_tutor_id}",
    operation_id = "removeCursusCompanyTutor",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("company_tutor_id" = mestier_core::CompanyTutorId, Path, description = "Company tutor record identifier"),
    ),
    responses(
        (status = 204, description = "The company tutor record was removed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Company tutor record not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CompanyTutorPath {
        organization_id,
        company_tutor_id,
    }: CompanyTutorPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .remove_company_tutor(RemoveCompanyTutorCommand {
            actor,
            organization_id,
            id: company_tutor_id,
        })
        .await?;

    Ok(Response::NoContent)
}
