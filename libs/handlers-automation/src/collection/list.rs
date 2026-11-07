use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use super::dto::CollectionSummaryResponse;
use crate::{paths::CollectionsPath, require_view_automation};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/collections",
    operation_id = "listCollections",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Every collection of this organization, without operations or schemas, sorted by title", body = inline(DataEnvelope<Vec<CollectionSummaryResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CollectionsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<CollectionSummaryResponse>>, ApiError> {
    require_view_automation(&state, &identity, path.organization_id).await?;

    let collections = state.usecase.list_collections(path.organization_id).await?;
    let body: Vec<CollectionSummaryResponse> = collections
        .into_iter()
        .map(CollectionSummaryResponse::from)
        .collect();

    Ok(Response::OK(body))
}
