use authz::Subject;
use axum::{Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateTicketCommand, CustomFields};

use crate::{
    intake::{
        IntakeOrganization, storable, storable_json,
        types::{CreateIntakeTicketRequest, IntakeTicketResponse},
        validation,
    },
    paths::IntakeTicketsPath,
};

const MAX_EXTERNAL_ID_CHARACTERS: usize = 255;

fn bounded_external_id(external_id: &str) -> Result<(), ApiError> {
    if external_id.chars().count() > MAX_EXTERNAL_ID_CHARACTERS {
        return Err(ApiError::UnprocessableEntity(format!(
            "external_id must be at most {MAX_EXTERNAL_ID_CHARACTERS} characters"
        )));
    }

    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/v1/support/intake/tickets",
    operation_id = "createIntakeTicket",
    tag = crate::TAG,
    description = "Creates a ticket in the organization that owns the API key. When external_id is already known in that organization the existing ticket is returned unchanged with a 200 instead of creating a second one.",
    request_body = CreateIntakeTicketRequest,
    responses(
        (status = 201, description = "The ticket was created", body = inline(DataEnvelope<IntakeTicketResponse>)),
        (status = 200, description = "A ticket with this external_id already exists in the organization; it is returned as is", body = inline(DataEnvelope<IntakeTicketResponse>)),
        (status = 401, description = "Missing, malformed, unknown or revoked API key"),
        (status = 422, description = "Blank subject or body, external_id longer than 255 characters, custom_fields that is not an object or exceeds its limits (64 keys, keys of 1 to 64 characters, 16 KiB)"),
        (status = 429, description = "Too many requests"),
    ),
    security(("intake_key" = []))
)]
pub async fn handler(
    _path: IntakeTicketsPath,
    IntakeOrganization(organization_id): IntakeOrganization,
    State(state): State<AppState>,
    Json(payload): Json<CreateIntakeTicketRequest>,
) -> Result<Response<IntakeTicketResponse>, ApiError> {
    storable(&payload.subject, "subject")?;
    storable(&payload.body, "body")?;
    if let Some(external_id) = &payload.external_id {
        storable(external_id, "external_id")?;
        bounded_external_id(external_id)?;
    }
    storable_json(&payload.custom_fields, "custom_fields")?;
    let custom_fields = CustomFields::try_from(payload.custom_fields).map_err(validation)?;
    if payload.body.trim().is_empty() {
        return Err(ApiError::UnprocessableEntity(
            "ticket body must not be empty".to_owned(),
        ));
    }

    let created = state
        .usecase
        .create_support_ticket(CreateTicketCommand {
            actor: Subject::system(),
            organization_id,
            subject: payload.subject,
            body: payload.body,
            custom_fields,
            external_id: payload.external_id,
        })
        .await
        .map_err(validation)?;

    let ticket = IntakeTicketResponse::from(created.ticket);

    Ok(if created.created {
        Response::Created(ticket)
    } else {
        Response::OK(ticket)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_external_id_of_255_characters_is_accepted() {
        assert!(bounded_external_id(&"a".repeat(255)).is_ok());
        assert!(bounded_external_id("autharie-ticket-4821").is_ok());
        assert!(bounded_external_id("").is_ok());
    }

    #[test]
    fn a_longer_external_id_is_unprocessable() {
        for length in [256, 3000] {
            assert!(
                matches!(
                    bounded_external_id(&"a".repeat(length)),
                    Err(ApiError::UnprocessableEntity(_))
                ),
                "{length}"
            );
        }
    }

    #[test]
    fn the_length_counts_characters_not_bytes() {
        assert!(bounded_external_id(&"é".repeat(255)).is_ok());
        assert!(bounded_external_id(&"\u{1F600}".repeat(255)).is_ok());
        assert!(matches!(
            bounded_external_id(&"é".repeat(256)),
            Err(ApiError::UnprocessableEntity(_))
        ));
    }

    #[test]
    fn the_message_names_the_field_and_the_limit_but_not_the_value() {
        let value = format!("secret{}", "a".repeat(300));

        match bounded_external_id(&value) {
            Err(ApiError::UnprocessableEntity(message)) => {
                assert!(message.contains("external_id"), "{message}");
                assert!(message.contains("255"), "{message}");
                assert!(!message.contains("secret"), "{message}");
            }
            other => panic!("{other:?}"),
        }
    }
}
