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
use serde::{Deserialize, Deserializer, de::Error};
use utoipa::IntoParams;

use crate::{
    paths::ParticipantCollectionPath, require_view_cursus, response::ParticipantListItemResponse,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListParticipantsQuery {
    #[param(inline)]
    pub kind: Option<ParticipantKind>,
    /// Comma-separated participant statuses (`pending`, `active`, `inactive`),
    /// for example `pending,active`. A participant matches when its status is
    /// any of them. Absent or empty means no filter.
    #[serde(default, deserialize_with = "comma_separated_statuses")]
    #[param(value_type = Option<String>, example = "pending,active")]
    pub status: Vec<ParticipantStatus>,
    pub host_company_id: Option<HostCompanyId>,
    /// A case-insensitive **substring** of the first name, the last name, `first last`, `last first` or the email. Not full-text
    /// search: no stemming, no ranking, no word boundaries. The value is
    /// trimmed, and `%`, `_` and `\` match themselves. Empty after trimming
    /// means no filter, which is what a cleared search box means.
    pub q: Option<String>,
}

fn comma_separated_statuses<'de, D>(deserializer: D) -> Result<Vec<ParticipantStatus>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    let mut statuses = Vec::new();
    for part in raw.as_deref().unwrap_or_default().split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let status = match part {
            "pending" => ParticipantStatus::Pending,
            "active" => ParticipantStatus::Active,
            "inactive" => ParticipantStatus::Inactive,
            other => return Err(D::Error::custom(format!("unknown status `{other}`"))),
        };
        if !statuses.contains(&status) {
            statuses.push(status);
        }
    }
    Ok(statuses)
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
