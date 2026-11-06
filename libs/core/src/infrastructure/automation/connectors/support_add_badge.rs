use std::str::FromStr;

use serde_json::{Map, Value, json};

use super::support_assign_ticket::ticket_id_from;
use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::{AddBadgeCommand, Badge, BadgeTone};

pub struct SupportAddBadgeConnector {
    usecase: MestierUseCase,
}

impl SupportAddBadgeConnector {
    pub fn new(usecase: MestierUseCase) -> Self {
        Self { usecase }
    }
}

fn tone_from(config: &Map<String, Value>) -> Result<BadgeTone, String> {
    match config.get("tone") {
        None | Some(Value::Null) => Ok(BadgeTone::default()),
        Some(Value::String(tone)) if tone.is_empty() => Ok(BadgeTone::default()),
        Some(Value::String(tone)) => BadgeTone::from_str(tone).map_err(|error| error.to_string()),
        Some(_) => Err("`tone` must resolve to a string".to_string()),
    }
}

fn badge_from(config: &Map<String, Value>) -> Result<Badge, String> {
    let label = config
        .get("label")
        .and_then(Value::as_str)
        .ok_or_else(|| "`label` must resolve to a string".to_string())?;
    let tone = tone_from(config)?;

    Badge::new(label, tone).map_err(|error| error.to_string())
}

impl Connector for SupportAddBadgeConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let ticket_id = match ticket_id_from(input.config) {
            Ok(ticket_id) => ticket_id,
            Err(error) => return ConnectorOutcome::Failed { error },
        };
        let badge = match badge_from(input.config) {
            Ok(badge) => badge,
            Err(error) => return ConnectorOutcome::Failed { error },
        };
        let output = json!({
            "ticket_id": ticket_id.0,
            "label": badge.label(),
            "tone": badge.tone().as_str(),
        });

        let usecase = self.usecase.acting_as_automation(input.run_id);

        match usecase
            .add_support_ticket_badge(AddBadgeCommand {
                actor: authz::Subject::system(),
                organization_id: input.org_id,
                ticket_id,
                badge,
            })
            .await
        {
            Ok(_) => ConnectorOutcome::Produced(output),
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
    use crate::infrastructure::realtime::EventHub;

    const A_UUID: &str = "018f3b2a-0000-7000-8000-000000000012";

    fn lazy_connector() -> SupportAddBadgeConnector {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        SupportAddBadgeConnector::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ))
    }

    fn config(ticket_id: Value, label: Value) -> Map<String, Value> {
        let mut config = Map::new();
        config.insert("ticket_id".to_string(), ticket_id);
        config.insert("label".to_string(), label);
        config
    }

    fn with_tone(mut config: Map<String, Value>, tone: Value) -> Map<String, Value> {
        config.insert("tone".to_string(), tone);
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
    async fn a_missing_ticket_id_fails_naming_the_field() {
        let mut config = config(json!(A_UUID), json!("High"));
        config.remove("ticket_id");

        assert_eq!(
            failure_of(&config).await,
            "`ticket_id` must resolve to a string"
        );
    }

    #[tokio::test]
    async fn a_ticket_id_that_is_not_a_uuid_fails_naming_the_field() {
        let error = failure_of(&config(json!("TCK-42"), json!("High"))).await;

        assert_eq!(error, "`ticket_id` must be a UUID");
    }

    #[tokio::test]
    async fn a_missing_or_non_string_label_fails_naming_the_field() {
        let mut missing = config(json!(A_UUID), json!("High"));
        missing.remove("label");

        assert_eq!(
            failure_of(&missing).await,
            "`label` must resolve to a string"
        );
        assert_eq!(
            failure_of(&config(json!(A_UUID), json!(5))).await,
            "`label` must resolve to a string"
        );
    }

    #[tokio::test]
    async fn a_blank_label_fails() {
        for label in ["", "   ", "\t\n"] {
            let error = failure_of(&config(json!(A_UUID), json!(label))).await;

            assert!(error.contains("badge label must not be empty"), "{error}");
        }
    }

    #[tokio::test]
    async fn an_overlong_label_fails() {
        let error = failure_of(&config(json!(A_UUID), json!("x".repeat(65)))).await;

        assert!(error.contains("at most 64 characters"), "{error}");
    }

    #[tokio::test]
    async fn an_unknown_tone_fails_naming_it() {
        let error = failure_of(&with_tone(
            config(json!(A_UUID), json!("High")),
            json!("loud"),
        ))
        .await;

        assert!(error.contains("unknown badge tone `loud`"), "{error}");
    }

    #[tokio::test]
    async fn a_tone_of_spaces_is_unknown_rather_than_defaulted() {
        let error = failure_of(&with_tone(
            config(json!(A_UUID), json!("High")),
            json!("  "),
        ))
        .await;

        assert!(error.contains("unknown badge tone"), "{error}");
    }

    #[tokio::test]
    async fn a_tone_that_is_not_a_string_fails_naming_the_field() {
        let error = failure_of(&with_tone(config(json!(A_UUID), json!("High")), json!(3))).await;

        assert_eq!(error, "`tone` must resolve to a string");
    }

    #[tokio::test]
    async fn a_failure_never_echoes_the_configuration() {
        let secret = "do-not-leak-this-token";

        let error = failure_of(&config(json!(secret), json!(secret))).await;

        assert!(!error.contains(secret), "{error}");
    }

    async fn run(
        world: &World,
        connector: &SupportAddBadgeConnector,
        config: &Map<String, Value>,
    ) -> ConnectorOutcome {
        connector
            .execute(input(world.organization_id, config))
            .await
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_tone_defaults_to_neutral_when_absent_null_or_empty() {
        let world = World::new(&pool().await).await;
        let connector = SupportAddBadgeConnector::new(world.usecase.clone());
        let ticket = world.ticket("T1").await;
        let id = json!(ticket.id.0.to_string());

        let absent = run(&world, &connector, &config(id.clone(), json!("High"))).await;
        let null = run(
            &world,
            &connector,
            &with_tone(config(id.clone(), json!("Null")), Value::Null),
        )
        .await;
        let empty = run(
            &world,
            &connector,
            &with_tone(config(id.clone(), json!("Empty")), json!("")),
        )
        .await;

        assert_eq!(
            absent,
            ConnectorOutcome::Produced(json!({
                "ticket_id": ticket.id.0.to_string(),
                "label": "High",
                "tone": "neutral",
            }))
        );
        assert!(matches!(null, ConnectorOutcome::Produced(_)), "{null:?}");
        assert!(matches!(empty, ConnectorOutcome::Produced(_)), "{empty:?}");
        let stored = world.get(ticket.id).await.unwrap();
        assert_eq!(
            stored.badges,
            vec![
                Badge::new("High", BadgeTone::Neutral).unwrap(),
                Badge::new("Null", BadgeTone::Neutral).unwrap(),
                Badge::new("Empty", BadgeTone::Neutral).unwrap(),
            ]
        );

        world.cleanup().await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_explicit_tone_is_stored_and_reported() {
        let world = World::new(&pool().await).await;
        let connector = SupportAddBadgeConnector::new(world.usecase.clone());
        let ticket = world.ticket("T1").await;

        let outcome = run(
            &world,
            &connector,
            &with_tone(
                config(json!(ticket.id.0.to_string()), json!("  Bloquant ")),
                json!("error"),
            ),
        )
        .await;

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(json!({
                "ticket_id": ticket.id.0.to_string(),
                "label": "Bloquant",
                "tone": "error",
            }))
        );
        let stored = world.get(ticket.id).await.unwrap();
        assert_eq!(
            stored.badges,
            vec![Badge::new("Bloquant", BadgeTone::Error).unwrap()]
        );

        world.cleanup().await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_unknown_tone_or_blank_label_adds_no_badge() {
        let world = World::new(&pool().await).await;
        let connector = SupportAddBadgeConnector::new(world.usecase.clone());
        let ticket = world.ticket("T1").await;
        let id = json!(ticket.id.0.to_string());

        let unknown_tone = run(
            &world,
            &connector,
            &with_tone(config(id.clone(), json!("High")), json!("loud")),
        )
        .await;
        let blank_label = run(&world, &connector, &config(id, json!("  "))).await;

        assert!(
            matches!(unknown_tone, ConnectorOutcome::Failed { .. }),
            "{unknown_tone:?}"
        );
        assert!(
            matches!(blank_label, ConnectorOutcome::Failed { .. }),
            "{blank_label:?}"
        );
        assert_eq!(world.get(ticket.id).await.unwrap(), ticket);

        world.cleanup().await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn adding_the_same_label_twice_leaves_one_badge() {
        let world = World::new(&pool().await).await;
        let connector = SupportAddBadgeConnector::new(world.usecase.clone());
        let ticket = world.ticket("T1").await;
        let id = json!(ticket.id.0.to_string());

        let first = run(&world, &connector, &config(id.clone(), json!("High"))).await;
        let again = run(&world, &connector, &config(id.clone(), json!(" High "))).await;
        let retoned = run(
            &world,
            &connector,
            &with_tone(config(id, json!("High")), json!("warning")),
        )
        .await;

        assert!(matches!(first, ConnectorOutcome::Produced(_)), "{first:?}");
        assert!(matches!(again, ConnectorOutcome::Produced(_)), "{again:?}");
        assert!(
            matches!(retoned, ConnectorOutcome::Produced(_)),
            "{retoned:?}"
        );
        let stored = world.get(ticket.id).await.unwrap();
        assert_eq!(
            stored.badges,
            vec![Badge::new("High", BadgeTone::Warning).unwrap()]
        );

        world.cleanup().await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_unknown_ticket_or_one_of_another_organization_fails() {
        let pool = pool().await;
        let acme = World::new(&pool).await;
        let globex = World::new(&pool).await;
        let connector = SupportAddBadgeConnector::new(acme.usecase.clone());
        let ticket = globex.ticket("Chez Globex").await;

        let unknown = run(
            &acme,
            &connector,
            &config(json!(Uuid::from_u128(0x99).to_string()), json!("High")),
        )
        .await;
        let foreign = run(
            &acme,
            &connector,
            &config(json!(ticket.id.0.to_string()), json!("High")),
        )
        .await;

        assert_eq!(
            unknown,
            ConnectorOutcome::Failed {
                error: "resource not found".to_string()
            }
        );
        assert_eq!(
            foreign,
            ConnectorOutcome::Failed {
                error: "resource not found".to_string()
            }
        );
        assert_eq!(globex.get(ticket.id).await.unwrap(), ticket);

        acme.cleanup().await;
        globex.cleanup().await;
    }
}
