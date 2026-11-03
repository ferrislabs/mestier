use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{ListTicketsCommand, MemberId, TicketFilter, TicketStatus};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    paths::TicketsPath, require_view_support, response::TicketResponse, ticket::parse_status,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListTicketsQuery {
    #[param(example = "new,in_progress")]
    pub status: Option<String>,
    pub assignee: Option<MemberId>,
    pub badge: Option<String>,
    pub custom_field_key: Option<String>,
    pub custom_field_value: Option<String>,
}

impl ListTicketsQuery {
    fn into_filter(self) -> Result<TicketFilter, ApiError> {
        let statuses = self
            .status
            .as_deref()
            .map(parse_statuses)
            .transpose()?
            .unwrap_or_default();
        let custom_field = match (self.custom_field_key, self.custom_field_value) {
            (Some(key), Some(value)) => Some((key, value)),
            (None, None) => None,
            _ => {
                return Err(ApiError::UnprocessableEntity(
                    "custom_field_key and custom_field_value must be given together".to_owned(),
                ));
            }
        };

        Ok(TicketFilter {
            statuses,
            assignee: self.assignee,
            badge: self.badge,
            custom_field,
        })
    }
}

fn parse_statuses(value: &str) -> Result<Vec<TicketStatus>, ApiError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(parse_status)
        .collect()
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/support/tickets",
    operation_id = "listSupportTickets",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListTicketsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "A page of tickets, newest first", body = inline(DataEnvelope<Vec<TicketResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Unknown status, or only one of custom_field_key and custom_field_value"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TicketsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ListTicketsQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<TicketResponse>, ApiError> {
    require_view_support(&state, &identity, path.organization_id).await?;
    let filter = query.into_filter()?;

    let per_page = pagination.per_page();
    let current_page = pagination.page();
    let page = state
        .usecase
        .list_support_tickets(ListTicketsCommand {
            organization_id: path.organization_id,
            filter,
            offset: pagination.offset(),
            limit: per_page,
        })
        .await?;

    let items: Vec<TicketResponse> = page.items.into_iter().map(TicketResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, current_page, Some(page.total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
