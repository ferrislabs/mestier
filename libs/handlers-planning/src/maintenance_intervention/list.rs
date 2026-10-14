use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    resolve_actor,
};
use mestier_core::InterventionStatus;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{
    maintenance_intervention::{InterventionResponse, InterventionsPath},
    require_org_membership,
};

#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum StatusFilter {
    #[default]
    Proposed,
    Validated,
    Rejected,
}

impl From<StatusFilter> for InterventionStatus {
    fn from(value: StatusFilter) -> Self {
        match value {
            StatusFilter::Proposed => Self::Proposed,
            StatusFilter::Validated => Self::Validated,
            StatusFilter::Rejected => Self::Rejected,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListInterventionsQuery {
    /// Defaults to `proposed`: the queue a manager works through.
    #[serde(default)]
    pub status: StatusFilter,
}

/// The organization's interventions in one status, oldest first.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/maintenance/interventions",
    operation_id = "listMaintenanceInterventions",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListInterventionsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated interventions", body = inline(DataEnvelope<Vec<InterventionResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InterventionsPath { organization_id }: InterventionsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(filter): Query<ListInterventionsQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<InterventionResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let (interventions, total) = state
        .usecase
        .list_maintenance_interventions(
            actor,
            organization_id,
            filter.status.into(),
            per_page,
            pagination.offset(),
        )
        .await?;

    let items: Vec<InterventionResponse> = interventions.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
