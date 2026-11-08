use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    resolve_actor,
};
use mestier_core::{ListAbsencesCommand, MemberId};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    absence::{AbsenceResponse, AbsencesPath},
    require_org_membership,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AbsencesQuery {
    pub member_id: Option<MemberId>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/absences",
    operation_id = "listAbsences",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        AbsencesQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated absences; a caller without planning or reference management only gets their own", body = inline(DataEnvelope<Vec<AbsenceResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Another member's absences need planning or reference management"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: AbsencesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<AbsencesQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<AbsenceResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let (absences, total) = state
        .usecase
        .acting_as(user_id)
        .list_absences_for(ListAbsencesCommand {
            actor,
            organization_id: path.organization_id,
            member_id: query.member_id,
            limit: per_page,
            offset,
        })
        .await?;
    let items: Vec<AbsenceResponse> = absences.into_iter().map(AbsenceResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
