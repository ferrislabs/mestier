use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    paths::ClosureReasonCollectionPath, require_view_cursus, response::ClosureReasonResponse,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListClosureReasonsQuery {
    #[param(example = false)]
    pub archived: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/closure-reasons",
    operation_id = "listCursusClosureReasons",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListClosureReasonsQuery,
    ),
    responses(
        (status = 200, description = "The closure reasons; archived rows only with `archived=true`", body = inline(DataEnvelope<Vec<ClosureReasonResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ClosureReasonCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListClosureReasonsQuery>,
) -> Result<Response<Vec<ClosureReasonResponse>>, ApiError> {
    require_view_cursus(&state, &identity, path.organization_id).await?;

    let items = state
        .usecase
        .list_closure_reasons(path.organization_id, query.archived.unwrap_or(false))
        .await?;

    Ok(Response::OK(
        items.into_iter().map(ClosureReasonResponse::from).collect(),
    ))
}
