use common::OrganizationId;
use serde_json::{Map, Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::TicketStatus;
use crate::application::MestierUseCase;
use crate::application::default_authorizer;
use crate::application::support_ticket::tests::{World, pool};
use crate::domain::automation::connector::{action_specs, connector_catalogue};
use crate::domain::automation::run::{ConnectorInput, ConnectorOutcome, DryRunInput};
use crate::infrastructure::automation::connectors::ConnectorRegistry;
use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
use crate::infrastructure::realtime::EventHub;

fn lazy_registry() -> ConnectorRegistry {
    let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("a lazy pool needs no server");
    ConnectorRegistry::new(MestierUseCase::new(
        pool,
        default_authorizer(),
        EventHub::new(),
    ))
}

fn config(pairs: &[(&str, Value)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
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

async fn run(
    registry: &ConnectorRegistry,
    kind: &str,
    organization_id: OrganizationId,
    config: &Map<String, Value>,
) -> ConnectorOutcome {
    registry
        .execute(kind, 1, input(organization_id, config))
        .await
        .unwrap()
}

fn produced(outcome: ConnectorOutcome) -> Value {
    match outcome {
        ConnectorOutcome::Produced(value) => value,
        other => panic!("expected an output, got {other:?}"),
    }
}

fn failure(outcome: ConnectorOutcome) -> String {
    match outcome {
        ConnectorOutcome::Failed { error } => error,
        other => panic!("expected a failure, got {other:?}"),
    }
}

fn organization() -> OrganizationId {
    OrganizationId(Uuid::from_u128(1))
}

#[tokio::test]
async fn every_described_action_has_a_runner_that_validates_before_touching_the_database() {
    let registry = lazy_registry();
    let invalid = config(&[("limit", json!("not a number"))]);

    for spec in action_specs() {
        let outcome = registry
            .execute(spec.kind, 1, input(organization(), &invalid))
            .await
            .unwrap_or_else(|_| panic!("`{}` has no runner", spec.kind));

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{}",
            spec.kind
        );
    }
}

#[tokio::test]
async fn an_action_of_an_unknown_version_is_refused() {
    let empty = Map::new();

    let error = lazy_registry()
        .execute(
            "mestier.support.get_ticket",
            2,
            input(organization(), &empty),
        )
        .await
        .unwrap_err();

    assert_eq!(error.version, 2);
}

#[tokio::test]
async fn an_invalid_config_fails_naming_the_field_without_echoing_it() {
    let registry = lazy_registry();
    let secret = "do-not-leak-this-token";
    let cases = [
        (
            "mestier.support.get_ticket",
            config(&[]),
            "`ticket_id` must resolve to a string",
        ),
        (
            "mestier.support.get_ticket",
            config(&[("ticket_id", json!(secret))]),
            "`ticket_id` must be a UUID",
        ),
        (
            "mestier.support.create_ticket",
            config(&[("subject", json!("  ")), ("body", json!("x"))]),
            "`subject` must not be blank",
        ),
        (
            "mestier.support.list_tickets",
            config(&[("custom_field_key", json!("k"))]),
            "`custom_field_key` and `custom_field_value` must be given together",
        ),
        (
            "mestier.support.list_tickets",
            config(&[("statuses", json!("new"))]),
            "`statuses` must resolve to an array",
        ),
        (
            "mestier.support.list_thread",
            config(&[
                ("ticket_id", json!(Uuid::from_u128(5).to_string())),
                ("client_visible_only", json!("maybe")),
            ]),
            "`client_visible_only` must be true or false",
        ),
        (
            "mestier.support.send_reply",
            config(&[
                ("ticket_id", json!(Uuid::from_u128(5).to_string())),
                ("body", json!(7)),
            ]),
            "`body` must resolve to a string",
        ),
        (
            "mestier.support.edit_draft",
            config(&[("entry_id", json!(secret)), ("body", json!("x"))]),
            "`entry_id` must be a UUID",
        ),
    ];

    for (kind, config, expected) in cases {
        let error = failure(run(&registry, kind, organization(), &config).await);

        assert_eq!(error, expected, "{kind}");
        assert!(!error.contains(secret));
    }
}

#[test]
fn the_dry_run_of_every_action_is_its_sample_and_never_reaches_the_service() {
    let catalogue = connector_catalogue();
    let empty = Map::new();

    for spec in action_specs() {
        let descriptor = catalogue.get(spec.kind, 1).unwrap();

        let outcome = ConnectorRegistry::dry_run(
            descriptor,
            &DryRunInput {
                config: &empty,
                stored: None,
            },
        );

        assert_eq!(
            outcome,
            Some(ConnectorOutcome::Produced(descriptor.sample_output.clone())),
            "{}",
            spec.kind
        );
    }
}

#[test]
fn a_field_of_an_action_is_a_plain_expression_input_except_a_select() {
    let catalogue = connector_catalogue();
    let transition = catalogue
        .get("mestier.support.transition_ticket", 1)
        .unwrap();

    assert_eq!(transition.group, Some("Support"));
    assert_eq!(transition.fields[0].name, "ticket_id");
    assert!(transition.fields[0].required);
    assert!(transition.fields[0].expression);
    assert_eq!(transition.fields[1].name, "status");
    assert!(!transition.fields[1].expression);
}

fn assert_same_keys(actual: &Value, kind: &str) {
    let sample = connector_catalogue()
        .get(kind, 1)
        .unwrap()
        .sample_output
        .clone();
    let sample = sample.get(0).cloned().unwrap_or(sample);
    let actual = actual.get(0).cloned().unwrap_or_else(|| actual.clone());
    let keys = |value: &Value| {
        let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    };
    assert_eq!(keys(&actual), keys(&sample), "{kind}");
}

fn id_of(value: &Value) -> String {
    value["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_ticket_is_read_listed_and_the_samples_name_what_a_run_returns() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Fuite").await;
    let registry = ConnectorRegistry::new(world.usecase.clone());
    let by_id = config(&[("ticket_id", json!(ticket.id.0.to_string()))]);

    let got = produced(
        run(
            &registry,
            "mestier.support.get_ticket",
            world.organization_id,
            &by_id,
        )
        .await,
    );
    let listed = produced(
        run(
            &registry,
            "mestier.support.list_tickets",
            world.organization_id,
            &config(&[("statuses", json!(["new"]))]),
        )
        .await,
    );

    assert_eq!(id_of(&got), ticket.id.0.to_string());
    assert_same_keys(&got, "mestier.support.get_ticket");
    assert_same_keys(&listed, "mestier.support.list_tickets");
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(id_of(&listed[0]), ticket.id.0.to_string());

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_ticket_is_created_and_transitioned_in_the_run_organization() {
    let world = World::new(&pool().await).await;
    let registry = ConnectorRegistry::new(world.usecase.clone());

    let created = produced(
        run(
            &registry,
            "mestier.support.create_ticket",
            world.organization_id,
            &config(&[
                ("subject", json!("Chaudière")),
                ("body", json!("En panne")),
                ("external_id", json!("ext-1")),
            ]),
        )
        .await,
    );
    let moved = produced(
        run(
            &registry,
            "mestier.support.transition_ticket",
            world.organization_id,
            &config(&[
                ("ticket_id", created["id"].clone()),
                ("status", json!("in_progress")),
            ]),
        )
        .await,
    );

    assert_eq!(created["created"], json!(true));
    assert_same_keys(&created, "mestier.support.create_ticket");
    assert_eq!(moved["status"], "in_progress");
    assert_same_keys(&moved, "mestier.support.transition_ticket");

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_object_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let ticket = acme.ticket("Chez Acme").await;
    let registry = ConnectorRegistry::new(acme.usecase.clone());
    let target = json!(ticket.id.0.to_string());
    let expected = "not found: the targeted object does not exist in this organization";

    for (kind, extra) in [
        ("mestier.support.get_ticket", vec![]),
        (
            "mestier.support.transition_ticket",
            vec![("status", json!("resolved"))],
        ),
        (
            "mestier.support.remove_badge",
            vec![("label", json!("High"))],
        ),
        ("mestier.support.list_thread", vec![]),
        (
            "mestier.support.send_reply",
            vec![("body", json!("Intrus"))],
        ),
        (
            "mestier.support.save_draft",
            vec![("body", json!("Intrus"))],
        ),
    ] {
        let mut pairs = vec![("ticket_id", target.clone())];
        pairs.extend(extra);
        let outcome = run(&registry, kind, globex.organization_id, &config(&pairs)).await;

        assert_eq!(failure(outcome), expected, "{kind}");
    }

    let other_member = json!(globex.manager.member_id.0.to_string());
    let assignee = run(
        &registry,
        "mestier.support.set_assignee",
        acme.organization_id,
        &config(&[
            ("ticket_id", target.clone()),
            ("assignee_member_id", other_member),
        ]),
    )
    .await;
    assert_eq!(failure(assignee), expected);

    assert_eq!(acme.get(ticket.id).await.unwrap().status, TicketStatus::New);
    let entries: i64 =
        sqlx::query_scalar("SELECT count(*) FROM support_ticket_entries WHERE ticket_id = $1")
            .bind(ticket.id.0)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(entries, 0);

    acme.cleanup().await;
    globex.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_field_naming_another_organization_is_ignored() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let ticket = acme.ticket("Chez Acme").await;
    let registry = ConnectorRegistry::new(acme.usecase.clone());
    let sneaky = config(&[
        ("organization_id", json!(acme.organization_id.0.to_string())),
        ("org_id", json!(acme.organization_id.0.to_string())),
    ]);

    let listed = produced(
        run(
            &registry,
            "mestier.support.list_tickets",
            globex.organization_id,
            &sneaky,
        )
        .await,
    );

    assert_eq!(listed, json!([]));
    assert_eq!(ticket.organization_id, acme.organization_id);

    acme.cleanup().await;
    globex.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_reply_and_a_draft_are_signed_by_the_automation_and_the_thread_shows_it() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Devis").await;
    let registry = ConnectorRegistry::new(world.usecase.clone());
    let target = json!(ticket.id.0.to_string());

    let draft = produced(
        run(
            &registry,
            "mestier.support.save_draft",
            world.organization_id,
            &config(&[("ticket_id", target.clone()), ("body", json!("Brouillon"))]),
        )
        .await,
    );
    let edited = produced(
        run(
            &registry,
            "mestier.support.edit_draft",
            world.organization_id,
            &config(&[
                ("entry_id", draft["id"].clone()),
                ("body", json!("Version finale")),
            ]),
        )
        .await,
    );
    let sent = produced(
        run(
            &registry,
            "mestier.support.send_draft",
            world.organization_id,
            &config(&[("entry_id", draft["id"].clone())]),
        )
        .await,
    );
    let reply = produced(
        run(
            &registry,
            "mestier.support.send_reply",
            world.organization_id,
            &config(&[("ticket_id", target.clone()), ("body", json!("Bonjour"))]),
        )
        .await,
    );
    let thread = produced(
        run(
            &registry,
            "mestier.support.list_thread",
            world.organization_id,
            &config(&[
                ("ticket_id", target.clone()),
                ("client_visible_only", json!(true)),
            ]),
        )
        .await,
    );

    assert_eq!(draft["kind"], "draft");
    assert_eq!(edited["body"], "Version finale");
    assert_eq!(sent["kind"], "reply");
    assert_eq!(sent["author_name"], "Automatisation");
    assert_eq!(reply["author_member_id"], Value::Null);
    assert_same_shape(&reply, "mestier.support.send_reply");
    assert_same_shape(&thread, "mestier.support.list_thread");
    assert_eq!(thread.as_array().unwrap().len(), 2);

    let deletable = produced(
        run(
            &registry,
            "mestier.support.save_draft",
            world.organization_id,
            &config(&[("ticket_id", target), ("body", json!("À jeter"))]),
        )
        .await,
    );
    let deleted = produced(
        run(
            &registry,
            "mestier.support.delete_draft",
            world.organization_id,
            &config(&[("entry_id", deletable["id"].clone())]),
        )
        .await,
    );
    assert_eq!(deleted["deleted"], json!(true));
    assert_same_shape(&deleted, "mestier.support.delete_draft");

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_dry_run_of_a_write_action_writes_nothing() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Intact").await;
    let catalogue = connector_catalogue();
    let descriptor = catalogue
        .get("mestier.support.transition_ticket", 1)
        .unwrap();
    let config = config(&[
        ("ticket_id", json!(ticket.id.0.to_string())),
        ("status", json!("resolved")),
    ]);

    let outcome = ConnectorRegistry::dry_run(
        descriptor,
        &DryRunInput {
            config: &config,
            stored: None,
        },
    );

    assert_eq!(
        outcome,
        Some(ConnectorOutcome::Produced(descriptor.sample_output.clone()))
    );
    assert_eq!(
        world.get(ticket.id).await.unwrap().status,
        TicketStatus::New
    );

    world.cleanup().await;
}
