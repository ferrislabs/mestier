use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::{AssignToLeastRecentCommand, MemberId, TicketId};

pub struct SupportAssignTicketConnector {
    usecase: MestierUseCase,
}

impl SupportAssignTicketConnector {
    pub fn new(usecase: MestierUseCase) -> Self {
        Self { usecase }
    }
}

pub(super) fn ticket_id_from(config: &Map<String, Value>) -> Result<TicketId, String> {
    let raw = config
        .get("ticket_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "`ticket_id` must resolve to a string".to_string())?;

    Uuid::parse_str(raw)
        .map(TicketId)
        .map_err(|_| "`ticket_id` must be a UUID".to_string())
}

fn members_from(config: &Map<String, Value>) -> Result<Vec<MemberId>, String> {
    let raw = config
        .get("members")
        .and_then(Value::as_array)
        .ok_or_else(|| "`members` must resolve to an array".to_string())?;

    if raw.is_empty() {
        return Err("`members` must not be empty".to_string());
    }

    raw.iter()
        .enumerate()
        .map(|(position, element)| {
            let text = element
                .as_str()
                .ok_or_else(|| format!("`members` element {position} must be a string"))?;
            Uuid::parse_str(text)
                .map(MemberId)
                .map_err(|_| format!("`members` element {position} must be a UUID"))
        })
        .collect()
}

impl Connector for SupportAssignTicketConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let ticket_id = match ticket_id_from(input.config) {
            Ok(ticket_id) => ticket_id,
            Err(error) => return ConnectorOutcome::Failed { error },
        };
        let members = match members_from(input.config) {
            Ok(members) => members,
            Err(error) => return ConnectorOutcome::Failed { error },
        };

        let usecase = self.usecase.acting_as_automation(input.run_id);

        match usecase
            .assign_support_ticket_to_least_recent(AssignToLeastRecentCommand {
                actor: authz::Subject::system(),
                organization_id: input.org_id,
                ticket_id,
                members,
            })
            .await
        {
            Ok(ticket) => ConnectorOutcome::Produced(json!({
                "ticket_id": ticket.id.0,
                "assignee_member_id": ticket.assignee.map(|member| member.0),
            })),
            Err(error) => ConnectorOutcome::Failed {
                error: error.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use common::OrganizationId;
    use sqlx::PgPool;

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::support_ticket::tests::{World, pool};
    use crate::infrastructure::realtime::EventHub;

    fn lazy_connector() -> SupportAssignTicketConnector {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        SupportAssignTicketConnector::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ))
    }

    fn config(ticket_id: Value, members: Value) -> Map<String, Value> {
        let mut config = Map::new();
        config.insert("ticket_id".to_string(), ticket_id);
        config.insert("members".to_string(), members);
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

    fn organization() -> OrganizationId {
        OrganizationId(Uuid::from_u128(1))
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
                .execute(input(organization(), config))
                .await,
        )
    }

    const A_UUID: &str = "018f3b2a-0000-7000-8000-000000000012";
    const ANOTHER_UUID: &str = "018f3b2a-0000-7000-8000-000000000013";

    #[tokio::test]
    async fn a_missing_ticket_id_fails_naming_the_field() {
        let mut config = config(json!(A_UUID), json!([A_UUID]));
        config.remove("ticket_id");

        let error = failure_of(&config).await;

        assert_eq!(error, "`ticket_id` must resolve to a string");
    }

    #[tokio::test]
    async fn a_non_string_ticket_id_fails_naming_the_field() {
        let error = failure_of(&config(json!(42), json!([A_UUID]))).await;

        assert_eq!(error, "`ticket_id` must resolve to a string");
    }

    #[tokio::test]
    async fn a_ticket_id_that_is_not_a_uuid_fails_naming_the_field() {
        let error = failure_of(&config(json!("TCK-42"), json!([A_UUID]))).await;

        assert_eq!(error, "`ticket_id` must be a UUID");
    }

    #[tokio::test]
    async fn a_missing_members_field_fails_naming_the_field() {
        let mut config = config(json!(A_UUID), json!([A_UUID]));
        config.remove("members");

        let error = failure_of(&config).await;

        assert_eq!(error, "`members` must resolve to an array");
    }

    #[tokio::test]
    async fn members_that_are_not_an_array_fail_naming_the_field() {
        for members in [
            json!(A_UUID),
            json!(12),
            json!({ "id": A_UUID }),
            Value::Null,
        ] {
            let error = failure_of(&config(json!(A_UUID), members)).await;

            assert_eq!(error, "`members` must resolve to an array");
        }
    }

    #[tokio::test]
    async fn an_empty_members_array_fails_naming_the_field() {
        let error = failure_of(&config(json!(A_UUID), json!([]))).await;

        assert_eq!(error, "`members` must not be empty");
    }

    #[tokio::test]
    async fn a_member_that_is_not_a_string_fails_naming_its_position() {
        let error = failure_of(&config(json!(A_UUID), json!([A_UUID, 7]))).await;

        assert_eq!(error, "`members` element 1 must be a string");
    }

    #[tokio::test]
    async fn a_member_that_is_not_a_uuid_fails_naming_its_position() {
        let error = failure_of(&config(json!(A_UUID), json!([A_UUID, ANOTHER_UUID, "bob"]))).await;

        assert_eq!(error, "`members` element 2 must be a UUID");
    }

    #[tokio::test]
    async fn every_kind_of_invalid_config_has_its_own_message() {
        let invalid = [
            config(json!(42), json!([A_UUID])),
            config(json!("TCK-42"), json!([A_UUID])),
            config(json!(A_UUID), json!("nobody")),
            config(json!(A_UUID), json!([])),
            config(json!(A_UUID), json!([7])),
            config(json!(A_UUID), json!(["bob"])),
        ];

        let mut messages = BTreeSet::new();
        for config in &invalid {
            messages.insert(failure_of(config).await);
        }

        assert_eq!(messages.len(), invalid.len(), "{messages:?}");
    }

    #[tokio::test]
    async fn a_failure_never_echoes_the_configuration() {
        let secret = "do-not-leak-this-token";

        let error = failure_of(&config(json!(secret), json!([secret]))).await;

        assert!(!error.contains(secret), "{error}");
        let error = failure_of(&config(json!(A_UUID), json!([secret]))).await;
        assert!(!error.contains(secret), "{error}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_invalid_config_assigns_nothing() {
        let world = World::new(&pool().await).await;
        let ticket = world.ticket("T1").await;
        let connector = SupportAssignTicketConnector::new(world.usecase.clone());
        let id = ticket.id.0.to_string();
        let member = world.plain.member_id.0.to_string();

        let invalid = [
            config(json!("TCK-42"), json!([member])),
            config(json!(id), json!("nobody")),
            config(json!(id), json!([])),
            config(json!(id), json!([member, 7])),
            config(json!(id), json!([member, "bob"])),
        ];
        for config in &invalid {
            let outcome = connector
                .execute(input(world.organization_id, config))
                .await;

            assert!(
                matches!(outcome, ConnectorOutcome::Failed { .. }),
                "{outcome:?}"
            );
        }

        assert_eq!(world.assigned_ticket_rows().await, 0);
        assert_eq!(world.get(ticket.id).await.unwrap(), ticket);

        world.cleanup().await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_valid_config_assigns_the_least_recent_member_and_reports_it() {
        let world = World::new(&pool().await).await;
        let connector = SupportAssignTicketConnector::new(world.usecase.clone());
        let first = world.ticket("T1").await;
        let second = world.ticket("T2").await;
        let third = world.ticket("T3").await;
        let members = json!([
            world.plain.member_id.0.to_string(),
            world.spare.member_id.0.to_string()
        ]);

        let to_first = connector
            .execute(input(
                world.organization_id,
                &config(json!(first.id.0.to_string()), members.clone()),
            ))
            .await;
        let to_second = connector
            .execute(input(
                world.organization_id,
                &config(json!(second.id.0.to_string()), members.clone()),
            ))
            .await;
        let to_third = connector
            .execute(input(
                world.organization_id,
                &config(json!(third.id.0.to_string()), members),
            ))
            .await;

        assert_eq!(
            to_first,
            ConnectorOutcome::Produced(json!({
                "ticket_id": first.id.0.to_string(),
                "assignee_member_id": world.plain.member_id.0.to_string(),
            }))
        );
        if let ConnectorOutcome::Produced(value) = &to_first {
            crate::infrastructure::automation::connectors::test_support::assert_same_shape(
                value,
                "mestier.support.assign_ticket",
            );
        }
        assert_eq!(
            to_second,
            ConnectorOutcome::Produced(json!({
                "ticket_id": second.id.0.to_string(),
                "assignee_member_id": world.spare.member_id.0.to_string(),
            }))
        );
        assert_eq!(
            to_third,
            ConnectorOutcome::Produced(json!({
                "ticket_id": third.id.0.to_string(),
                "assignee_member_id": world.plain.member_id.0.to_string(),
            }))
        );
        let stored = world.get(first.id).await.unwrap();
        assert_eq!(stored.assignee, Some(world.plain.member_id));
        assert!(stored.assigned_at.is_some());

        world.cleanup().await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_member_of_another_organization_or_an_unknown_ticket_fails_and_assigns_nothing() {
        let pool = pool().await;
        let acme = World::new(&pool).await;
        let globex = World::new(&pool).await;
        let ticket = acme.ticket("Chez Acme").await;
        let connector = SupportAssignTicketConnector::new(acme.usecase.clone());

        let foreign = connector
            .execute(input(
                acme.organization_id,
                &config(
                    json!(ticket.id.0.to_string()),
                    json!([globex.plain.member_id.0.to_string()]),
                ),
            ))
            .await;
        let unknown = connector
            .execute(input(
                acme.organization_id,
                &config(
                    json!(Uuid::from_u128(0x99).to_string()),
                    json!([acme.plain.member_id.0.to_string()]),
                ),
            ))
            .await;
        let of_the_other_organization = connector
            .execute(input(
                globex.organization_id,
                &config(
                    json!(ticket.id.0.to_string()),
                    json!([globex.plain.member_id.0.to_string()]),
                ),
            ))
            .await;

        assert!(
            matches!(foreign, ConnectorOutcome::Failed { .. }),
            "{foreign:?}"
        );
        assert!(
            matches!(unknown, ConnectorOutcome::Failed { .. }),
            "{unknown:?}"
        );
        assert!(
            matches!(of_the_other_organization, ConnectorOutcome::Failed { .. }),
            "{of_the_other_organization:?}"
        );
        assert_eq!(acme.assigned_ticket_rows().await, 0);
        assert_eq!(globex.assigned_ticket_rows().await, 0);
        assert_eq!(acme.get(ticket.id).await.unwrap(), ticket);

        acme.cleanup().await;
        globex.cleanup().await;
    }
}
