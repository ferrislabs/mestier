use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{SchoolTutorFilter, SearchPattern};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{paths::SchoolTutorCollectionPath, require_view_cursus, response::SchoolTutorResponse};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListSchoolTutorsQuery {
    /// Which rows to return by archive state: `active` (the default) keeps
    /// school tutors with no `archived_at`, `archived` keeps only archived
    /// ones, `all` keeps both.
    #[param(inline, example = "active")]
    pub archived: Option<mestier_core::ArchivedFilter>,
    /// A case-insensitive **substring** of the member's name (last, first, or
    /// both in either order) or email. The value is trimmed, and `%`, `_` and
    /// `\` match themselves. Empty after trimming means no filter.
    pub q: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/school-tutors",
    operation_id = "listCursusSchoolTutors",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListSchoolTutorsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "A page of school tutors narrowed by archive state and `q`, ordered by name; `total` counts the filtered rows", body = inline(DataEnvelope<Vec<SchoolTutorResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: SchoolTutorCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListSchoolTutorsQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<SchoolTutorResponse>, ApiError> {
    require_view_cursus(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let current_page = pagination.page();
    let (items, total) = state
        .usecase
        .list_school_tutors(
            path.organization_id,
            SchoolTutorFilter {
                archived: query.archived.unwrap_or_default(),
                search: SearchPattern::parse(query.q.as_deref()),
            },
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<SchoolTutorResponse> =
        items.into_iter().map(SchoolTutorResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, current_page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
