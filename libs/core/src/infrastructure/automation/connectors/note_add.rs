use common::CoreError;
use serde_json::{Map, Value, json};

use super::fields;
use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::{AddEntryCommand, TicketId};

pub const MAX_NOTE_CHARS: usize = 10_000;

pub struct NoteAddConnector {
    usecase: MestierUseCase,
}

impl NoteAddConnector {
    pub fn new(usecase: MestierUseCase) -> Self {
        Self { usecase }
    }
}

fn note_from(config: &Map<String, Value>) -> Result<(TicketId, String), String> {
    let ticket_id = fields::uuid(config, "ticket_id").map(TicketId)?;
    let body = fields::filled(config, "body", MAX_NOTE_CHARS)?;
    Ok((ticket_id, body.to_owned()))
}

impl Connector for NoteAddConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let (ticket_id, body) = match note_from(input.config) {
            Ok(note) => note,
            Err(error) => return ConnectorOutcome::Failed { error },
        };

        let usecase = self.usecase.acting_as_automation(input.run_id);

        match usecase
            .add_support_automation_note(AddEntryCommand {
                actor: authz::Subject::system(),
                organization_id: input.org_id,
                ticket_id,
                body,
            })
            .await
        {
            Ok(entry) => ConnectorOutcome::Produced(json!({
                "ticket_id": entry.ticket_id.0,
                "note_id": entry.id.0,
            })),
            Err(CoreError::NotFound) => ConnectorOutcome::Failed {
                error: "`ticket_id` does not match a ticket of this organization".to_owned(),
            },
            Err(error) => ConnectorOutcome::Failed {
                error: error.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use common::OrganizationId;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::support_ticket::tests::{World, pool};
    use crate::domain::automation::connector::connector_catalogue;
    use crate::domain::automation::run::DryRunInput;
    use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
    use crate::infrastructure::realtime::EventHub;

    const A_UUID: &str = "018f3b2a-0000-7000-8000-000000000012";

    fn lazy_connector() -> NoteAddConnector {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        NoteAddConnector::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ))
    }

    fn config(ticket_id: Value, body: Value) -> Map<String, Value> {
        let mut config = Map::new();
        config.insert("ticket_id".to_owned(), ticket_id);
        config.insert("body".to_owned(), body);
        config
    }

    fn input(organization_id: OrganizationId, config: &Map<String, Value>) -> ConnectorInput<'_> {
        ConnectorInput {
            org_id: organization_id,
            run_id: Uuid::from_u128(2),
            config,
            credential_id: None,
            operation: None,
        }
    }

    fn failure(outcome: ConnectorOutcome) -> String {
        match outcome {
            ConnectorOutcome::Failed { error } => error,
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    async fn failure_of(config: &Map<String, Value>) -> String {
        failure(
            lazy_connector()
                .execute(input(OrganizationId(Uuid::from_u128(1)), config))
                .await,
        )
    }

    #[tokio::test]
    async fn an_invalid_config_fails_naming_the_field_without_echoing_it() {
        let secret = "do-not-leak-this-token";
        let cases = [
            (
                config(json!(42), json!("note")),
                "`ticket_id` must resolve to a string",
            ),
            (
                config(json!(secret), json!("note")),
                "`ticket_id` must be a UUID",
            ),
            (
                config(json!(A_UUID), json!(7)),
                "`body` must resolve to a string",
            ),
            (
                config(json!(A_UUID), json!("  ")),
                "`body` must not be blank",
            ),
            (
                config(json!(A_UUID), json!("x".repeat(MAX_NOTE_CHARS + 1))),
                "`body` holds at most 10000 characters",
            ),
        ];

        for (config, expected) in cases {
            let error = failure_of(&config).await;

            assert_eq!(error, expected);
            assert!(!error.contains(secret));
        }
    }

    #[tokio::test]
    async fn a_missing_field_fails_naming_it() {
        let mut without_body = config(json!(A_UUID), json!("note"));
        without_body.remove("body");

        assert_eq!(
            failure_of(&without_body).await,
            "`body` must resolve to a string"
        );
    }

    #[test]
    fn the_dry_run_answers_with_the_sample_and_the_sample_names_what_a_run_returns() {
        let catalogue = connector_catalogue();
        let descriptor = catalogue.get("mestier.note.add", 1).unwrap();
        let config = config(json!(A_UUID), json!("note"));

        let outcome = NoteAddConnector::dry_run(
            descriptor,
            &DryRunInput {
                config: &config,
                stored: None,
            },
        );

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(descriptor.sample_output.clone())
        );
        assert_same_shape(
            &json!({ "ticket_id": A_UUID, "note_id": A_UUID }),
            "mestier.note.add",
        );
    }

    async fn notes_of(pool: &PgPool, ticket_id: Uuid) -> Vec<(String, Option<Uuid>, String)> {
        sqlx::query_as(
            "SELECT kind, author_member_id, body FROM support_ticket_entries
             WHERE ticket_id = $1 ORDER BY created_at",
        )
        .bind(ticket_id)
        .fetch_all(pool)
        .await
        .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_note_is_added_to_a_ticket_of_the_run_organization() {
        let world = World::new(&pool().await).await;
        let ticket = world.ticket("T1").await;
        let connector = NoteAddConnector::new(world.usecase.clone());
        let config = config(json!(ticket.id.0.to_string()), json!("Relance faite"));

        let outcome = connector
            .execute(input(world.organization_id, &config))
            .await;

        let ConnectorOutcome::Produced(output) = outcome else {
            panic!("expected an output, got {outcome:?}");
        };
        assert_eq!(output["ticket_id"], ticket.id.0.to_string());
        assert_same_shape(&output, "mestier.note.add");
        let notes = notes_of(&world.pool, ticket.id.0).await;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].0, "internal_note");
        assert_eq!(notes[0].1, None);
        assert_eq!(notes[0].2, "Relance faite");
        assert_eq!(output["note_id"].as_str().unwrap().len(), 36);

        world.cleanup().await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_ticket_of_another_organization_is_refused_and_nothing_is_written() {
        let pool = pool().await;
        let acme = World::new(&pool).await;
        let globex = World::new(&pool).await;
        let ticket = acme.ticket("Chez Acme").await;
        let connector = NoteAddConnector::new(acme.usecase.clone());
        let config = config(json!(ticket.id.0.to_string()), json!("Intrus"));

        let from_globex = connector
            .execute(input(globex.organization_id, &config))
            .await;
        let unknown = connector
            .execute(input(
                acme.organization_id,
                &self::config(json!(Uuid::from_u128(0x99).to_string()), json!("x")),
            ))
            .await;

        assert_eq!(
            failure(from_globex),
            "`ticket_id` does not match a ticket of this organization"
        );
        assert_eq!(
            failure(unknown),
            "`ticket_id` does not match a ticket of this organization"
        );
        assert!(notes_of(&pool, ticket.id.0).await.is_empty());

        acme.cleanup().await;
        globex.cleanup().await;
    }
}
