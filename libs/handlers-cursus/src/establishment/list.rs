use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    paths::EstablishmentCollectionPath, require_view_cursus, response::EstablishmentResponse,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListEstablishmentsQuery {
    #[param(example = false)]
    pub archived: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/establishments",
    operation_id = "listCursusEstablishments",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListEstablishmentsQuery,
    ),
    responses(
        (status = 200, description = "The establishments; archived rows only with `archived=true`", body = inline(DataEnvelope<Vec<EstablishmentResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EstablishmentCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListEstablishmentsQuery>,
) -> Result<Response<Vec<EstablishmentResponse>>, ApiError> {
    require_view_cursus(&state, &identity, path.organization_id).await?;

    let items = state
        .usecase
        .list_establishments(path.organization_id, query.archived.unwrap_or(false))
        .await?;

    Ok(Response::OK(
        items.into_iter().map(EstablishmentResponse::from).collect(),
    ))
}
