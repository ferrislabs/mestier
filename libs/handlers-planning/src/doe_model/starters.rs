use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::StarterModel;

use crate::{
    doe_model::{DoeModelStartersPath, response::DoeModelStarterResponse},
    require_org_membership,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/doe-model-starters",
    operation_id = "listDoeModelStarters",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Starter models shipped with Mestier", body = inline(DataEnvelope<Vec<DoeModelStarterResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DoeModelStartersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<DoeModelStarterResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    Ok(Response::OK(
        StarterModel::all()
            .iter()
            .copied()
            .map(Into::into)
            .collect(),
    ))
}
