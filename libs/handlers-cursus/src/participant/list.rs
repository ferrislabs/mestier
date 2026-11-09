use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{ParticipantKind, ParticipantStatus};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{paths::ParticipantCollectionPath, require_view_cursus, response::ParticipantResponse};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListParticipantsQuery {
    #[param(inline)]
    pub kind: Option<ParticipantKind>,
    #[param(inline)]
    pub status: Option<ParticipantStatus>,
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
        (status = 200, description = "A page of participants, filtered by kind and status", body = inline(DataEnvelope<Vec<ParticipantResponse>>)),
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
) -> Result<Response<ParticipantResponse>, ApiError> {
    require_view_cursus(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let current_page = pagination.page();
    let (items, total) = state
        .usecase
        .list_participants(
            path.organization_id,
            query.kind,
            query.status,
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<ParticipantResponse> =
        items.into_iter().map(ParticipantResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, current_page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
