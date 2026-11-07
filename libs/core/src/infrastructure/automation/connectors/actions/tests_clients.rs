use common::OrganizationId;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::application::MestierUseCase;
use crate::application::default_authorizer;
use crate::application::test_support::automation_pool;
use crate::domain::automation::connector::connector_catalogue;
use crate::domain::automation::run::{ConnectorInput, ConnectorOutcome, DryRunInput};
use crate::infrastructure::automation::connectors::ConnectorRegistry;
use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
use crate::infrastructure::automation::connectors::world_support;
use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
use crate::infrastructure::realtime::EventHub;

pub(in crate::infrastructure::automation::connectors::actions) const NOT_FOUND: &str =
    "not found: the targeted object does not exist in this organization";

pub(in crate::infrastructure::automation::connectors::actions) fn registry(
    pool: &sqlx::PgPool,
) -> ConnectorRegistry {
    ConnectorRegistry::new(MestierUseCase::new(
        pool.clone(),
        default_authorizer(),
        EventHub::new(),
    ))
}

pub(in crate::infrastructure::automation::connectors::actions) fn config(
    pairs: &[(&str, Value)],
) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

pub(in crate::infrastructure::automation::connectors::actions) async fn run(
    registry: &ConnectorRegistry,
    kind: &str,
    organization_id: OrganizationId,
    config: &Map<String, Value>,
) -> ConnectorOutcome {
    registry
        .execute(
            kind,
            1,
            ConnectorInput {
                org_id: organization_id,
                run_id: Uuid::from_u128(2),
                config,
                credential_id: None,
                operation: None,
            },
        )
        .await
        .unwrap()
}

pub(in crate::infrastructure::automation::connectors::actions) fn produced(
    outcome: ConnectorOutcome,
) -> Value {
    match outcome {
        ConnectorOutcome::Produced(value) => value,
        other => panic!("expected an output, got {other:?}"),
    }
}

pub(in crate::infrastructure::automation::connectors::actions) fn failure(
    outcome: ConnectorOutcome,
) -> String {
    match outcome {
        ConnectorOutcome::Failed { error } => error,
        other => panic!("expected a failure, got {other:?}"),
    }
}

async fn count(pool: &sqlx::PgPool, sql: &str, id: Uuid) -> i64 {
    world_support::count(pool, sql, id).await
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_customer_is_read_and_listed_in_the_shape_the_catalogue_samples() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "clients-read").await;
    let customer = world_support::customer(&pool, acme).await;
    let registry = registry(&pool);

    let got = produced(
        run(
            &registry,
            "mestier.customer.get",
            acme,
            &config(&[("customer_id", json!(customer.id.to_string()))]),
        )
        .await,
    );
    let listed = produced(run(&registry, "mestier.customer.list", acme, &config(&[])).await);
    let contexts = produced(
        run(
            &registry,
            "mestier.customer_context.list",
            acme,
            &config(&[("customer_id", json!(customer.id.to_string()))]),
        )
        .await,
    );

    assert_eq!(got["id"], json!(customer.id.to_string()));
    assert_same_shape(&got, "mestier.customer.get");
    assert_same_shape(&listed, "mestier.customer.list");
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_same_shape(&contexts, "mestier.customer_context.list");
    assert_eq!(contexts[0]["id"], json!(customer.context_id.to_string()));
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn contacts_contexts_and_customers_are_written_in_the_run_organization() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "clients-write").await;
    let customer = world_support::customer(&pool, acme).await;
    let registry = registry(&pool);
    let target = json!(customer.id.to_string());

    let updated = produced(
        run(
            &registry,
            "mestier.customer.update",
            acme,
            &config(&[
                ("customer_id", target.clone()),
                ("status", json!("CLIENT")),
                ("name", json!("Claire Fontaine SARL")),
            ]),
        )
        .await,
    );
    let contact = produced(
        run(
            &registry,
            "mestier.customer_contact.create",
            acme,
            &config(&[
                ("customer_id", target.clone()),
                ("first_name", json!("Paul")),
                ("last_name", json!("Durand")),
                ("is_primary", json!(true)),
            ]),
        )
        .await,
    );
    let renamed = produced(
        run(
            &registry,
            "mestier.customer_contact.update",
            acme,
            &config(&[
                ("contact_id", contact["id"].clone()),
                ("first_name", json!("Pauline")),
                ("last_name", json!("Durand")),
            ]),
        )
        .await,
    );
    let place = produced(
        run(
            &registry,
            "mestier.customer_context.create",
            acme,
            &config(&[
                ("customer_id", target.clone()),
                ("label", json!("Résidence")),
            ]),
        )
        .await,
    );
    let deleted = produced(
        run(
            &registry,
            "mestier.customer_contact.delete",
            acme,
            &config(&[("contact_id", contact["id"].clone())]),
        )
        .await,
    );

    assert_eq!(updated["status"], "CLIENT");
    assert_same_shape(&updated, "mestier.customer.update");
    assert_same_shape(&contact, "mestier.customer_contact.create");
    assert_eq!(renamed["first_name"], "Pauline");
    assert_eq!(place["label"], "Résidence");
    assert_same_shape(&place, "mestier.customer_context.create");
    assert_eq!(deleted["deleted"], json!(true));
    assert_same_shape(&deleted, "mestier.customer_contact.delete");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_object_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "clients-acme").await;
    let globex = world_support::organization(&pool, "clients-globex").await;
    let customer = world_support::customer(&pool, acme).await;
    let registry = registry(&pool);
    let target = json!(customer.id.to_string());
    let place = json!(customer.context_id.to_string());

    let cases = [
        (
            "mestier.customer.get",
            vec![("customer_id", target.clone())],
        ),
        (
            "mestier.customer.update",
            vec![
                ("customer_id", target.clone()),
                ("status", json!("ARCHIVED")),
                ("name", json!("Intrus")),
            ],
        ),
        (
            "mestier.customer.delete",
            vec![("customer_id", target.clone())],
        ),
        (
            "mestier.customer_contact.create",
            vec![
                ("customer_id", target.clone()),
                ("first_name", json!("Intrus")),
                ("last_name", json!("Intrus")),
            ],
        ),
        (
            "mestier.customer_context.get",
            vec![("context_id", place.clone())],
        ),
        (
            "mestier.customer_context.delete",
            vec![("context_id", place.clone())],
        ),
        (
            "mestier.customer_context.list",
            vec![("customer_id", target.clone())],
        ),
    ];

    for (kind, pairs) in cases {
        let outcome = run(&registry, kind, globex, &config(&pairs)).await;

        assert_eq!(failure(outcome), NOT_FOUND, "{kind}");
    }

    let name: String = sqlx::query_scalar("SELECT name FROM customers WHERE id = $1")
        .bind(customer.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(name, "Claire Fontaine");
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM customers WHERE id = $1 AND deleted_at IS NULL",
            customer.id
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM customer_contacts WHERE customer_id = $1",
            customer.id
        )
        .await,
        0
    );
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_dry_run_of_a_write_action_writes_nothing() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "clients-dry").await;
    let customer = world_support::customer(&pool, acme).await;
    let catalogue = connector_catalogue();
    let descriptor = catalogue.get("mestier.customer.delete", 1).unwrap();
    let config = config(&[("customer_id", json!(customer.id.to_string()))]);

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
        count(
            &pool,
            "SELECT count(*) FROM customers WHERE id = $1 AND deleted_at IS NULL",
            customer.id
        )
        .await,
        1
    );
}
