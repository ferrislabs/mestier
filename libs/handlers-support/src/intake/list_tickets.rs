use axum::extract::{Query, State};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{ListTicketsCommand, TicketFilter};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    intake::{IntakeOrganization, storable, types::IntakeTicketResponse},
    paths::IntakeTicketsPath,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListIntakeTicketsQuery {
    #[param(example = "plan")]
    pub custom_field_key: Option<String>,
    #[param(example = "pro")]
    pub custom_field_value: Option<String>,
}

impl ListIntakeTicketsQuery {
    fn into_filter(self) -> Result<TicketFilter, ApiError> {
        let custom_field = match (self.custom_field_key, self.custom_field_value) {
            (Some(key), Some(value)) => {
                storable(&key, "custom_field_key")?;
                storable(&value, "custom_field_value")?;
                Some((key, value))
            }
            (None, None) => None,
            _ => {
                return Err(ApiError::UnprocessableEntity(
                    "custom_field_key and custom_field_value must be given together".to_owned(),
                ));
            }
        };

        Ok(TicketFilter {
            custom_field,
            ..TicketFilter::default()
        })
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/support/intake/tickets",
    operation_id = "listIntakeTickets",
    tag = crate::TAG,
    description = "Lists the tickets of the organization that owns the API key, newest first, without their entries. custom_field_key and custom_field_value must be given together; the value of the top-level key is compared as text.",
    params(
        ListIntakeTicketsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "A page of tickets, newest first", body = inline(DataEnvelope<Vec<IntakeTicketResponse>>)),
        (status = 401, description = "Missing, malformed, unknown or revoked API key"),
        (status = 422, description = "Only one of custom_field_key and custom_field_value was given"),
        (status = 429, description = "Too many requests"),
    ),
    security(("intake_key" = []))
)]
pub async fn handler(
    _path: IntakeTicketsPath,
    IntakeOrganization(organization_id): IntakeOrganization,
    State(state): State<AppState>,
    Query(query): Query<ListIntakeTicketsQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<IntakeTicketResponse>, ApiError> {
    let filter = query.into_filter()?;

    let per_page = pagination.per_page();
    let current_page = pagination.page();
    let page = state
        .usecase
        .list_support_tickets(ListTicketsCommand {
            organization_id,
            filter,
            offset: (current_page - 1).saturating_mul(per_page),
            limit: per_page,
        })
        .await?;

    let items: Vec<IntakeTicketResponse> = page
        .items
        .into_iter()
        .map(IntakeTicketResponse::from)
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, current_page, Some(page.total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(key: Option<&str>, value: Option<&str>) -> ListIntakeTicketsQuery {
        ListIntakeTicketsQuery {
            custom_field_key: key.map(str::to_owned),
            custom_field_value: value.map(str::to_owned),
        }
    }

    #[test]
    fn both_parameters_make_one_filter() {
        let filter = query(Some("plan"), Some("42")).into_filter().unwrap();

        assert_eq!(
            filter,
            TicketFilter {
                custom_field: Some(("plan".to_owned(), "42".to_owned())),
                ..TicketFilter::default()
            }
        );
    }

    #[test]
    fn no_parameter_makes_no_filter() {
        assert_eq!(
            query(None, None).into_filter().unwrap(),
            TicketFilter::default()
        );
    }

    #[test]
    fn a_nul_character_in_either_parameter_is_unprocessable() {
        for query in [
            query(Some("a\0b"), Some("42")),
            query(Some("plan"), Some("a\0b")),
        ] {
            assert!(matches!(
                query.into_filter(),
                Err(ApiError::UnprocessableEntity(_))
            ));
        }
    }

    #[test]
    fn one_parameter_alone_is_unprocessable() {
        for query in [query(Some("plan"), None), query(None, Some("42"))] {
            assert!(matches!(
                query.into_filter(),
                Err(ApiError::UnprocessableEntity(_))
            ));
        }
    }
}
