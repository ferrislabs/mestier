use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{
    HostCompanyId, ParticipantFilter, ParticipantKind, ParticipantStatus, SearchPattern,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    paths::ParticipantCollectionPath, require_view_cursus, response::ParticipantListItemResponse,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListParticipantsQuery {
    #[param(inline)]
    pub kind: Option<ParticipantKind>,
    #[param(inline)]
    pub status: Option<ParticipantStatus>,
    pub host_company_id: Option<HostCompanyId>,
    /// A case-insensitive **substring** of the first name, the last name, `first last`, `last first` or the email. Not full-text
    /// search: no stemming, no ranking, no word boundaries. The value is
    /// trimmed, and `%`, `_` and `\` match themselves. Empty after trimming
    /// means no filter, which is what a cleared search box means.
    pub q: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/cursus/participants",
    operation_id = "listCursusParticipants",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListParticipantsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "A page of participants, filtered by kind, status, host company and `q` (`total` counts the filtered rows), with their apprentice and company tutor records", body = inline(DataEnvelope<Vec<ParticipantListItemResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ParticipantCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListParticipantsQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<ParticipantListItemResponse>, ApiError> {
    require_view_cursus(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let current_page = pagination.page();
    let (items, total) = state
        .usecase
        .list_participants(
            path.organization_id,
            ParticipantFilter {
                kind: query.kind,
                status: query.status,
                host_company_id: query.host_company_id,
                search: SearchPattern::parse(query.q.as_deref()),
            },
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<ParticipantListItemResponse> = items
        .into_iter()
        .map(ParticipantListItemResponse::from)
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, current_page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
