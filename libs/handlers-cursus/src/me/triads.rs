use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};

use crate::{paths::MyTriadCollectionPath, response::MyTriadResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/me/triads",
    operation_id = "listMyCursusTriads",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The caller's triads with their role and access", body = inline(DataEnvelope<Vec<MyTriadResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "The caller is not an active participant of the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: MyTriadCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<MyTriadResponse>>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;

    let triads = state
        .usecase
        .list_my_triads(user_id, path.organization_id)
        .await?;

    Ok(Response::OK(
        triads.into_iter().map(MyTriadResponse::from).collect(),
    ))
}
