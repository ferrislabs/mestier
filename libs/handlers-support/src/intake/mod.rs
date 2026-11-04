use axum::{Router, extract::DefaultBodyLimit};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState};
use mestier_core::{ListThreadCommand, OrganizationId, TicketEntry, TicketId};
use serde_json::Value;

pub mod attach_file;
pub mod attachment_url;
pub mod create_ticket;
pub mod get_ticket;
pub mod guard;
pub mod list_tickets;
pub mod post_message;
pub mod types;
pub mod upload;

pub use guard::IntakeOrganization;

pub fn router(state: &AppState) -> Router<AppState> {
    let max_upload_bytes =
        usize::try_from(state.args.file_storage.max_upload_bytes).unwrap_or(usize::MAX);

    let uploads = Router::new()
        .typed_post(attach_file::handler)
        .layer(DefaultBodyLimit::max(max_upload_bytes));

    Router::new()
        .typed_post(create_ticket::handler)
        .typed_get(list_tickets::handler)
        .typed_get(get_ticket::handler)
        .typed_post(post_message::handler)
        .typed_get(attachment_url::handler)
        .merge(uploads)
}

pub(crate) fn validation(error: impl Into<ApiError>) -> ApiError {
    match error.into() {
        ApiError::Conflict(message) => ApiError::UnprocessableEntity(message),
        other => other,
    }
}

pub(crate) fn storable(text: &str, field: &str) -> Result<(), ApiError> {
    if text.contains('\0') {
        return Err(ApiError::UnprocessableEntity(format!(
            "{field} must not contain a NUL character"
        )));
    }

    Ok(())
}

pub(crate) fn storable_json(value: &Value, field: &str) -> Result<(), ApiError> {
    let holds_nul = match value {
        Value::String(text) => text.contains('\0'),
        Value::Array(items) => items.iter().any(|item| storable_json(item, field).is_err()),
        Value::Object(map) => map
            .iter()
            .any(|(key, item)| key.contains('\0') || storable_json(item, field).is_err()),
        _ => false,
    };
    if holds_nul {
        return Err(ApiError::UnprocessableEntity(format!(
            "{field} must not contain a NUL character"
        )));
    }

    Ok(())
}

pub(crate) async fn client_visible_thread(
    state: &AppState,
    organization_id: OrganizationId,
    ticket_id: TicketId,
) -> Result<Vec<TicketEntry>, ApiError> {
    let entries = state
        .usecase
        .list_support_thread(ListThreadCommand {
            organization_id,
            ticket_id,
            client_visible_only: true,
        })
        .await?;

    Ok(entries
        .into_iter()
        .filter(|entry| entry.kind.is_client_visible())
        .collect())
}

#[cfg(test)]
mod tests {
    use mestier_core::CustomFields;
    use serde_json::json;

    use super::*;

    #[test]
    fn text_with_a_nul_character_is_not_storable() {
        assert!(storable("plain text", "subject").is_ok());
        assert!(storable("héllo \u{1F600}", "subject").is_ok());
        assert!(matches!(
            storable("a\0b", "subject"),
            Err(ApiError::UnprocessableEntity(message)) if message.contains("subject")
        ));
    }

    #[test]
    fn a_nul_character_is_found_at_any_depth_of_a_json_value() {
        assert!(
            storable_json(
                &json!({ "a": [1, { "b": "fine" }], "c": null }),
                "custom_fields"
            )
            .is_ok()
        );
        for value in [
            json!("a\0b"),
            json!(["a\0b"]),
            json!({ "a": "a\0b" }),
            json!({ "a\0b": 1 }),
            json!({ "a": [{ "b": [["a\0b"]] }] }),
        ] {
            assert!(
                matches!(
                    storable_json(&value, "custom_fields"),
                    Err(ApiError::UnprocessableEntity(_))
                ),
                "{value}"
            );
        }
    }

    #[test]
    fn a_domain_validation_conflict_becomes_a_422() {
        let error = validation(CustomFields::try_from(json!(["team"])).unwrap_err());

        assert!(matches!(error, ApiError::UnprocessableEntity(_)));
    }

    #[test]
    fn a_conflict_keeps_its_message_as_a_422() {
        let error = validation(ApiError::Conflict("must not be empty".to_owned()));

        assert!(
            matches!(error, ApiError::UnprocessableEntity(message) if message == "must not be empty")
        );
    }

    #[test]
    fn anything_but_a_conflict_keeps_its_status() {
        assert!(matches!(validation(ApiError::NotFound), ApiError::NotFound));
        assert!(matches!(
            validation(ApiError::Forbidden),
            ApiError::Forbidden
        ));
    }
}
