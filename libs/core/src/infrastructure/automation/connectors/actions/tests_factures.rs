use serde_json::json;
use uuid::Uuid;

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
        "quantity": 10,
        "unit_price_cents": 10000,
        "vat_rate_basis_points": 2000,
    }])
}

async fn project(pool: &sqlx::PgPool, org: common::OrganizationId) -> Uuid {
    let id = common::generate_uuid_v7();
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(org.0)
        .bind("Chantier")
        .execute(pool)
        .await
        .unwrap();
    id
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_invoice_is_created_read_listed_and_balanced() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "factures-write").await;
    let customer = world_support::customer(&pool, acme).await;
    let registry = registry(&pool);

    let created = produced(
        run(
            &registry,
            "mestier.invoice.create",
            acme,
            &config(&[
                ("kind", json!("STANDARD")),
                ("customer_id", json!(customer.id.to_string())),
                (
                    "customer_context_id",
                    json!(customer.context_id.to_string()),
                ),
                ("operation_nature", json!("SERVICES")),
                ("lines", lines()),
            ]),
        )
        .await,
    );
    let target = config(&[("invoice_id", created["id"].clone())]);
    let got = produced(run(&registry, "mestier.invoice.get", acme, &target).await);
    let listed = produced(run(&registry, "mestier.invoice.list", acme, &config(&[])).await);
    let balance = produced(run(&registry, "mestier.invoice.balance", acme, &target).await);
    let payments = produced(run(&registry, "mestier.invoice.list_payments", acme, &target).await);
    let outstanding = produced(
        run(
            &registry,
            "mestier.invoice.outstanding_by_customer",
            acme,
            &config(&[]),
        )
        .await,
    );

    assert_eq!(created["net_cents"], 100000);
    assert_eq!(created["status"], "DRAFT");
    assert_eq!(got["id"], created["id"]);
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_same_shape(&balance, "mestier.invoice.balance");
    assert_eq!(balance["remaining_cents"], 100000);
    assert_eq!(payments, json!([]));
    assert!(outstanding.is_array());
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_invoice_or_project_of_another_organization_fails_with_a_named_error() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "factures-acme").await;
    let globex = world_support::organization(&pool, "factures-globex").await;
    let customer = world_support::customer(&pool, acme).await;
    let registry = registry(&pool);
    let created = produced(
        run(
            &registry,
            "mestier.invoice.create",
            acme,
            &config(&[
                ("kind", json!("STANDARD")),
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
    let invoice = created["id"].clone();
    let foreign_project = json!(project(&pool, acme).await.to_string());

    let cases = [
        ("mestier.invoice.get", vec![("invoice_id", invoice.clone())]),
        (
            "mestier.invoice.issue",
            vec![("invoice_id", invoice.clone())],
        ),
        (
            "mestier.invoice.cancel",
            vec![("invoice_id", invoice.clone())],
        ),
        (
            "mestier.invoice.balance",
            vec![("invoice_id", invoice.clone())],
        ),
        (
            "mestier.invoice.issue_credit_note",
            vec![("invoice_id", invoice.clone()), ("lines", lines())],
        ),
        (
            "mestier.invoice.project_billing_summary",
            vec![("project_id", foreign_project.clone())],
        ),
        (
            "mestier.invoice.issue_deposit",
            vec![
                ("project_id", foreign_project.clone()),
                ("percentage_bp", json!(3000)),
            ],
        ),
        (
            "mestier.invoice.create",
            vec![
                ("kind", json!("STANDARD")),
                ("customer_id", json!(customer.id.to_string())),
                (
                    "customer_context_id",
                    json!(customer.context_id.to_string()),
                ),
                ("lines", lines()),
            ],
        ),
    ];

    for (kind, pairs) in cases {
        let outcome = run(&registry, kind, globex, &config(&pairs)).await;

        assert_eq!(failure(outcome), NOT_FOUND, "{kind}");
    }

    let status: String = sqlx::query_scalar("SELECT status::text FROM invoices WHERE id = $1")
        .bind(Uuid::parse_str(invoice.as_str().unwrap()).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "DRAFT");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_dry_run_of_a_write_action_writes_nothing() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "factures-dry").await;
    let customer = world_support::customer(&pool, acme).await;
    let registry = registry(&pool);
    let created = produced(
        run(
            &registry,
            "mestier.invoice.create",
            acme,
            &config(&[
                ("kind", json!("STANDARD")),
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
    let catalogue = connector_catalogue();
    let descriptor = catalogue.get("mestier.invoice.cancel", 1).unwrap();
    let config = config(&[("invoice_id", created["id"].clone())]);

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
    let status: String = sqlx::query_scalar("SELECT status::text FROM invoices WHERE id = $1")
        .bind(Uuid::parse_str(created["id"].as_str().unwrap()).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "DRAFT");
}

async fn issued_invoice(
    registry: &crate::infrastructure::automation::connectors::ConnectorRegistry,
    pool: &sqlx::PgPool,
    org: common::OrganizationId,
) -> serde_json::Value {
    sqlx::query(
        "UPDATE organizations SET legal_name = 'Treyve Paysages', legal_form = 'SARL', \
         registration_number = '123456789', insurance_mention = 'Assurance décennale', \
         vat_status = 'not_subject', vat_exemption_basis = 'Art. 293 B du CGI', \
         address_line1 = '12 rue des Lilas', address_postal_code = '75001', \
         address_city = 'Paris', address_country = 'FR' WHERE id = $1",
    )
    .bind(org.0)
    .execute(pool)
    .await
    .unwrap();
    let customer = world_support::customer(pool, org).await;
    let created = produced(
        run(
            registry,
            "mestier.invoice.create",
            org,
            &config(&[
                ("kind", json!("STANDARD")),
                ("customer_id", json!(customer.id.to_string())),
                (
                    "customer_context_id",
                    json!(customer.context_id.to_string()),
                ),
                ("operation_nature", json!("SERVICES")),
                ("lines", lines()),
            ]),
        )
        .await,
    );
    produced(
        run(
            registry,
            "mestier.invoice.issue",
            org,
            &config(&[("invoice_id", created["id"].clone())]),
        )
        .await,
    )
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_payment_is_recorded_by_the_system_user_listed_and_deleted() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "factures-payment").await;
    let registry = registry(&pool);
    let invoice = issued_invoice(&registry, &pool, acme).await;

    let payment = produced(
        run(
            &registry,
            "mestier.invoice.record_payment",
            acme,
            &config(&[
                ("invoice_id", invoice["id"].clone()),
                ("amount_cents", json!(40000)),
                ("paid_on", json!("2026-01-15")),
                ("method", json!("virement")),
            ]),
        )
        .await,
    );
    let target = config(&[("invoice_id", invoice["id"].clone())]);
    let listed = produced(run(&registry, "mestier.invoice.list_payments", acme, &target).await);
    let author: String = sqlx::query_scalar("SELECT sub FROM users WHERE id = $1")
        .bind(Uuid::parse_str(payment["recorded_by"].as_str().unwrap()).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_same_shape(&payment, "mestier.invoice.record_payment");
    assert_eq!(author, "system:automation");
    assert_eq!(listed.as_array().unwrap().len(), 1);

    let deleted = produced(
        run(
            &registry,
            "mestier.invoice.delete_payment",
            acme,
            &config(&[("payment_id", payment["id"].clone())]),
        )
        .await,
    );
    let after = produced(run(&registry, "mestier.invoice.list_payments", acme, &target).await);

    assert_eq!(deleted["deleted"], true);
    assert_eq!(after, json!([]));
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_payment_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "factures-pay-acme").await;
    let globex = world_support::organization(&pool, "factures-pay-globex").await;
    let registry = registry(&pool);
    let invoice = issued_invoice(&registry, &pool, acme).await;
    let payment = produced(
        run(
            &registry,
            "mestier.invoice.record_payment",
            acme,
            &config(&[
                ("invoice_id", invoice["id"].clone()),
                ("amount_cents", json!(1000)),
                ("paid_on", json!("2026-01-15")),
                ("method", json!("virement")),
            ]),
        )
        .await,
    );

    let record = run(
        &registry,
        "mestier.invoice.record_payment",
        globex,
        &config(&[
            ("invoice_id", invoice["id"].clone()),
            ("amount_cents", json!(1000)),
            ("paid_on", json!("2026-01-15")),
            ("method", json!("virement")),
        ]),
    )
    .await;
    let delete = run(
        &registry,
        "mestier.invoice.delete_payment",
        globex,
        &config(&[("payment_id", payment["id"].clone())]),
    )
    .await;

    assert_eq!(failure(record), NOT_FOUND);
    assert_eq!(failure(delete), NOT_FOUND);
    let remaining = produced(
        run(
            &registry,
            "mestier.invoice.list_payments",
            acme,
            &config(&[("invoice_id", invoice["id"].clone())]),
        )
        .await,
    );
    assert_eq!(remaining.as_array().unwrap().len(), 1);
}
