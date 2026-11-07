use chrono::NaiveDate;
use common::{OrganizationId, generate_uuid_v7};
use rust_decimal::Decimal;
use serde_json::{Map, Value, json};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::application::test_support::scratch_pool;
use crate::application::{MestierUseCase, default_authorizer};
use crate::domain::automation::connector::connector_catalogue;
use crate::domain::automation::run::{ConnectorInput, ConnectorOutcome, DryRunInput};
use crate::domain::supplier_invoice::commands::{
    CreateSupplierInvoiceCommand, SupplierInvoiceLineCommand,
};
use crate::infrastructure::automation::connectors::ConnectorRegistry;
use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
use crate::infrastructure::realtime::EventHub;
use crate::{SupplierInvoice, SupplierInvoiceSource};

static SCRATCH: OnceCell<String> = OnceCell::const_new();

const NOT_FOUND: &str = "not found: the targeted object does not exist in this organization";

struct World {
    usecase: MestierUseCase,
    pool: PgPool,
    registry: ConnectorRegistry,
    organization_id: OrganizationId,
}

impl World {
    async fn new(pool: &PgPool) -> Self {
        let owner = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(owner)
        .bind(format!("owner-{owner}@example.com"))
        .bind(format!("owner-{owner}"))
        .bind("Owner")
        .bind(format!("sub-{owner}"))
        .execute(pool)
        .await
        .unwrap();
        let organization_id = OrganizationId(generate_uuid_v7());
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id.0)
            .bind("Treyve Paysages")
            .bind(format!("org-{}", organization_id.0))
            .bind(owner)
            .execute(pool)
            .await
            .unwrap();
        let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
        Self {
            pool: pool.clone(),
            registry: ConnectorRegistry::new(usecase.clone()),
            usecase,
            organization_id,
        }
    }

    async fn run(&self, kind: &str, pairs: &[(&str, Value)]) -> ConnectorOutcome {
        self.run_as(self.organization_id, kind, pairs).await
    }

    async fn run_as(
        &self,
        organization_id: OrganizationId,
        kind: &str,
        pairs: &[(&str, Value)],
    ) -> ConnectorOutcome {
        let config: Map<String, Value> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect();
        self.registry
            .execute(
                kind,
                1,
                ConnectorInput {
                    org_id: organization_id,
                    run_id: Uuid::from_u128(2),
                    config: &config,
                    credential_id: None,
                    operation: None,
                },
            )
            .await
            .unwrap()
    }

    async fn ok(&self, kind: &str, pairs: &[(&str, Value)]) -> Value {
        match self.run(kind, pairs).await {
            ConnectorOutcome::Produced(value) => value,
            other => panic!("{kind}: expected an output, got {other:?}"),
        }
    }
}

impl World {
    async fn invoice(&self, number: &str) -> SupplierInvoice {
        self.usecase
            .create_supplier_invoice(CreateSupplierInvoiceCommand {
                organization_id: self.organization_id,
                supplier_name: "Point P".to_owned(),
                supplier_registration_number: None,
                supplier_vat_number: None,
                number: number.to_owned(),
                issued_on: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                due_on: None,
                source: SupplierInvoiceSource::Manual,
                currency: "EUR".to_owned(),
                lines: vec![SupplierInvoiceLineCommand {
                    label: "Sable".to_owned(),
                    quantity: Decimal::from(2),
                    unit: None,
                    unit_price_cents: 22_500,
                    line_total_cents: 45_000,
                    vat_rate_basis_points: Some(2000),
                }],
                source_file_key: None,
                source_file_mime_type: None,
            })
            .await
            .unwrap()
    }

    async fn project(&self) -> Uuid {
        let id = generate_uuid_v7();
        sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(self.organization_id.0)
            .bind("Jardin")
            .execute(&self.pool)
            .await
            .unwrap();
        id
    }
}

fn failed(outcome: ConnectorOutcome) -> String {
    match outcome {
        ConnectorOutcome::Failed { error } => error,
        other => panic!("expected a failure, got {other:?}"),
    }
}

async fn pool() -> PgPool {
    scratch_pool("mestier_lot9b_achats_", &SCRATCH).await
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_supplier_is_created_read_updated_and_listed() {
    let world = World::new(&pool().await).await;

    let created = world
        .ok(
            "mestier.achats.create_supplier",
            &[("name", json!("Point P"))],
        )
        .await;
    let got = world
        .ok(
            "mestier.achats.get_supplier",
            &[("supplier_id", created["id"].clone())],
        )
        .await;
    let updated = world
        .ok(
            "mestier.achats.update_supplier",
            &[
                ("supplier_id", created["id"].clone()),
                ("name", json!("Point P Nord")),
            ],
        )
        .await;
    let listed = world.ok("mestier.achats.list_suppliers", &[]).await;

    assert_eq!(got["name"], "Point P");
    assert_eq!(updated["name"], "Point P Nord");
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_same_shape(&created, "mestier.achats.create_supplier");
    assert_same_shape(&listed, "mestier.achats.list_suppliers");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_invoice_is_read_annotated_confirmed_and_costs_follow_the_allocations() {
    let world = World::new(&pool().await).await;
    let invoice = world.invoice("F-1").await;
    let id = json!(invoice.id.0.to_string());
    let line = json!(invoice.lines[0].id.0.to_string());
    let project = world.project().await;

    let got = world
        .ok(
            "mestier.achats.get_supplier_invoice",
            &[("supplier_invoice_id", id.clone())],
        )
        .await;
    let other = world.invoice("F-2").await;
    let noted = world
        .ok(
            "mestier.achats.update_supplier_invoice_notes",
            &[
                ("supplier_invoice_id", json!(other.id.0.to_string())),
                ("notes", json!("Vu")),
            ],
        )
        .await;
    let allocated = world
        .ok(
            "mestier.achats.replace_line_allocations",
            &[
                ("supplier_invoice_line_id", line),
                (
                    "allocations",
                    json!([{ "project_id": project.to_string(), "amount_cents": 45000 }]),
                ),
            ],
        )
        .await;
    let costs = world
        .ok(
            "mestier.achats.get_project_supplier_costs",
            &[("project_id", json!(project.to_string()))],
        )
        .await;
    let confirmed = world
        .ok(
            "mestier.achats.confirm_supplier_invoice",
            &[("supplier_invoice_id", id)],
        )
        .await;
    let listed = world.ok("mestier.achats.list_supplier_invoices", &[]).await;

    assert_eq!(noted["notes"], "Vu");
    assert_eq!(allocated.as_array().unwrap().len(), 1);
    assert_eq!(costs["allocated_cents"], 45000);
    assert_eq!(confirmed["status"], "CONFIRMED");
    assert_eq!(listed.as_array().unwrap().len(), 2);
    assert_same_shape(&got, "mestier.achats.get_supplier_invoice");
    assert_same_shape(&allocated, "mestier.achats.replace_line_allocations");
    assert_same_shape(&costs, "mestier.achats.get_project_supplier_costs");
    assert_same_shape(&confirmed, "mestier.achats.confirm_supplier_invoice");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_object_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let supplier = acme
        .ok(
            "mestier.achats.create_supplier",
            &[("name", json!("Chez Acme"))],
        )
        .await;
    let invoice = acme.invoice("F-ACME").await;
    let project = acme.project().await;
    let invoice_id = json!(invoice.id.0.to_string());
    let line = json!(invoice.lines[0].id.0.to_string());

    let attempts = [
        (
            "mestier.achats.get_supplier",
            vec![("supplier_id", supplier["id"].clone())],
        ),
        (
            "mestier.achats.update_supplier",
            vec![
                ("supplier_id", supplier["id"].clone()),
                ("name", json!("X")),
            ],
        ),
        (
            "mestier.achats.get_supplier_invoice",
            vec![("supplier_invoice_id", invoice_id.clone())],
        ),
        (
            "mestier.achats.confirm_supplier_invoice",
            vec![("supplier_invoice_id", invoice_id.clone())],
        ),
        (
            "mestier.achats.reject_supplier_invoice",
            vec![("supplier_invoice_id", invoice_id.clone())],
        ),
        (
            "mestier.achats.update_supplier_invoice_notes",
            vec![
                ("supplier_invoice_id", invoice_id.clone()),
                ("notes", json!("X")),
            ],
        ),
        (
            "mestier.achats.replace_line_allocations",
            vec![
                ("supplier_invoice_line_id", line),
                ("allocations", json!([])),
            ],
        ),
        (
            "mestier.achats.get_project_supplier_costs",
            vec![("project_id", json!(project.to_string()))],
        ),
    ];

    for (kind, pairs) in attempts {
        let outcome = acme.run_as(globex.organization_id, kind, &pairs).await;
        assert_eq!(failed(outcome), NOT_FOUND, "{kind}");
    }

    let got = acme
        .ok(
            "mestier.achats.get_supplier_invoice",
            &[("supplier_invoice_id", invoice_id)],
        )
        .await;
    assert_eq!(got["status"], "RECEIVED");
    assert_eq!(got["notes"], Value::Null);
    let others = globex.ok("mestier.achats.list_suppliers", &[]).await;
    assert_eq!(others, json!([]));
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_dry_run_of_a_write_action_writes_nothing() {
    let world = World::new(&pool().await).await;
    let catalogue = connector_catalogue();
    let descriptor = catalogue.get("mestier.achats.create_supplier", 1).unwrap();
    let config: Map<String, Value> = [("name".to_owned(), json!("Fantôme"))]
        .into_iter()
        .collect();

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
        world.ok("mestier.achats.list_suppliers", &[]).await,
        json!([])
    );
}
