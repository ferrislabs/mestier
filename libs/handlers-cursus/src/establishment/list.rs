use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{EstablishmentFilter, SearchPattern};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    paths::EstablishmentCollectionPath, require_view_cursus, response::EstablishmentResponse,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListEstablishmentsQuery {
    /// Which rows to return by archive state: `active` (the default) keeps
    /// rows with no `archived_at`, `archived` keeps only archived rows, `all`
    /// keeps both.
    #[param(inline, example = "active")]
    pub archived: Option<mestier_core::ArchivedFilter>,
    /// A case-insensitive **substring** of the name or the UAI. Not full-text
    /// search: no stemming, no ranking, no word boundaries. The value is
    /// trimmed, and `%`, `_` and `\` match themselves. Empty after trimming
    /// means no filter, which is what a cleared search box means.
    pub q: Option<String>,
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
        (status = 200, description = "The establishments narrowed by archive state and `q`", body = inline(DataEnvelope<Vec<EstablishmentResponse>>)),
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
        .list_establishments(
            path.organization_id,
            EstablishmentFilter {
                archived: query.archived.unwrap_or_default(),
                search: SearchPattern::parse(query.q.as_deref()),
            },
        )
        .await?;

    Ok(Response::OK(
        items.into_iter().map(EstablishmentResponse::from).collect(),
    ))
}
