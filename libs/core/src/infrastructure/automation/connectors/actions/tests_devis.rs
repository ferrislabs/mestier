use serde_json::json;

use super::super::clients::tests_clients::{NOT_FOUND, config, failure, produced, registry, run};
use crate::application::test_support::automation_pool;
use crate::domain::automation::connector::connector_catalogue;
use crate::domain::automation::run::{ConnectorOutcome, DryRunInput};
use crate::infrastructure::automation::connectors::ConnectorRegistry;
use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
use crate::infrastructure::automation::connectors::world_support;
use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;

fn lines() -> serde_json::Value {
    json!([{
        "label": "Pose de lames",
        "quantity": "10",
        "unit": "m²",
        "unit_price_cents": 10000,
        "vat_rate_bp": 2000,
    }])
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_quote_is_created_read_and_moved_to_another_status() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "devis-write").await;
    let customer = world_support::customer_billed_in_nantes(&pool, acme).await;
    let registry = registry(&pool);

    let created = produced(
        run(
            &registry,
            "mestier.quote.create",
            acme,
            &config(&[
                ("title", json!("Terrasse")),
                ("customer_id", json!(customer.id.to_string())),
                (
                    "customer_context_id",
                    json!(customer.context_id.to_string()),
                ),
                ("lines", lines()),
            ]),
        )
        .await,
    );
    let got = produced(
        run(
            &registry,
            "mestier.quote.get",
            acme,
            &config(&[("quote_id", created["id"].clone())]),
        )
        .await,
    );
    let sent = produced(
        run(
            &registry,
            "mestier.quote.update_status",
            acme,
            &config(&[
                ("quote_id", created["id"].clone()),
                ("status", json!("SENT")),
            ]),
        )
        .await,
    );

    assert_eq!(created["title"], "Terrasse");
    assert_eq!(created["net_cents"], 100000);
    assert_same_shape(&sent, "mestier.quote.update_status");
    assert_eq!(got["id"], created["id"]);
    assert_eq!(sent["status"], "SENT");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_quote_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "devis-acme").await;
    let globex = world_support::organization(&pool, "devis-globex").await;
    let customer = world_support::customer(&pool, acme).await;
    let quote = world_support::quote(&pool, acme, &customer, "DRAFT", "Chez Acme", 1).await;
    let registry = registry(&pool);
    let target = json!(quote.to_string());
    let customer_id = json!(customer.id.to_string());
    let context_id = json!(customer.context_id.to_string());

    let cases = [
        ("mestier.quote.get", vec![("quote_id", target.clone())]),
        (
            "mestier.quote.update_status",
            vec![("quote_id", target.clone()), ("status", json!("ACCEPTED"))],
        ),
        ("mestier.quote.delete", vec![("quote_id", target.clone())]),
        (
            "mestier.quote.plan_proposal",
            vec![("quote_id", target.clone())],
        ),
        (
            "mestier.quote.create",
            vec![
                ("title", json!("Intrus")),
                ("customer_id", customer_id.clone()),
                ("customer_context_id", context_id.clone()),
                ("lines", lines()),
            ],
        ),
    ];

    for (kind, pairs) in cases {
        let outcome = run(&registry, kind, globex, &config(&pairs)).await;

        assert_eq!(failure(outcome), NOT_FOUND, "{kind}");
    }

    let status: String = sqlx::query_scalar("SELECT status::text FROM quotes WHERE id = $1")
        .bind(quote)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "DRAFT");
    assert_eq!(
        world_support::count(
            &pool,
            "SELECT count(*) FROM quotes WHERE org_id = $1",
            globex.0
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
    let acme = world_support::organization(&pool, "devis-dry").await;
    let customer = world_support::customer(&pool, acme).await;
    let quote = world_support::quote(&pool, acme, &customer, "DRAFT", "Intact", 1).await;
    let catalogue = connector_catalogue();
    let descriptor = catalogue.get("mestier.quote.delete", 1).unwrap();
    let config = config(&[("quote_id", json!(quote.to_string()))]);

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
        world_support::count(
            &pool,
            "SELECT count(*) FROM quotes WHERE id = $1 AND deleted_at IS NULL",
            quote
        )
        .await,
        1
    );
}
