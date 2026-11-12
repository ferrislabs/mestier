use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{SearchPattern, TrainingFilter};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{paths::TrainingCollectionPath, require_view_cursus, response::TrainingResponse};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListTrainingsQuery {
    pub establishment_id: Option<mestier_core::EstablishmentId>,
    /// Which rows to return by archive state: `active` (the default) keeps
    /// rows with no `archived_at`, `archived` keeps only archived rows, `all`
    /// keeps both.
    #[param(inline, example = "active")]
    pub archived: Option<mestier_core::ArchivedFilter>,
    /// A case-insensitive **substring** of the name or the RNCP code. Not full-text
    /// search: no stemming, no ranking, no word boundaries. The value is
    /// trimmed, and `%`, `_` and `\` match themselves. Empty after trimming
    /// means no filter, which is what a cleared search box means.
    pub q: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/trainings",
    operation_id = "listCursusTrainings",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListTrainingsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "A page of trainings narrowed by establishment, archive state and `q`; `total` counts the filtered rows", body = inline(DataEnvelope<Vec<TrainingResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TrainingCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListTrainingsQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<TrainingResponse>, ApiError> {
    require_view_cursus(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let current_page = pagination.page();
    let (items, total) = state
        .usecase
        .list_trainings(
            path.organization_id,
            TrainingFilter {
                establishment_id: query.establishment_id,
                archived: query.archived.unwrap_or_default(),
                search: SearchPattern::parse(query.q.as_deref()),
            },
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<TrainingResponse> = items.into_iter().map(TrainingResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, current_page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
