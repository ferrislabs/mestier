use common::{OrganizationId, generate_uuid_v7};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use super::super::clients::tests_clients::{NOT_FOUND, config, failure, produced, registry, run};
use crate::application::test_support::automation_pool;
use crate::domain::automation::connector::connector_catalogue;
use crate::domain::automation::run::{ConnectorOutcome, DryRunInput};
use crate::infrastructure::automation::connectors::ConnectorRegistry;
use crate::infrastructure::automation::connectors::test_support::sample_of;
use crate::infrastructure::automation::connectors::world_support;
use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;

fn same_keys(actual: &Value, kind: &str) {
    let sample = sample_of(kind);
    let (actual, sample) = match (actual.as_array(), sample.as_array()) {
        (Some(items), Some(samples)) => (items.first().unwrap_or(&Value::Null), &samples[0]),
        _ => (actual, &sample),
    };
    let keys = |value: &Value| {
        let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    };
    assert_eq!(keys(actual), keys(sample), "{kind}");
}

async fn member(pool: &PgPool, organization: OrganizationId) -> Uuid {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, $3)",
    )
    .bind(id)
    .bind(organization.0)
    .bind("Alice Employee")
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn count(pool: &PgPool, sql: &str, id: Uuid) -> i64 {
    world_support::count(pool, sql, id).await
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_catalogue_is_read_and_listed_in_the_shape_the_samples_promise() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "ref-read").await;
    let registry = registry(&pool);

    let product = produced(
        run(
            &registry,
            "mestier.product.create",
            acme,
            &config(&[
                ("name", json!("Radiateur")),
                ("unit", json!("UNIT")),
                ("unit_price_cents", json!(24900)),
            ]),
        )
        .await,
    );
    let got = produced(
        run(
            &registry,
            "mestier.product.get",
            acme,
            &config(&[("product_id", product["id"].clone())]),
        )
        .await,
    );
    let listed = produced(run(&registry, "mestier.product.list", acme, &config(&[])).await);
    let units = produced(run(&registry, "mestier.custom_unit.list", acme, &config(&[])).await);

    assert_eq!(got["name"], "Radiateur");
    same_keys(&product, "mestier.product.create");
    same_keys(&got, "mestier.product.get");
    same_keys(&listed, "mestier.product.list");
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(units, json!([]));
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn reference_data_is_written_in_the_run_organization() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "ref-write").await;
    let registry = registry(&pool);

    let rate = produced(
        run(
            &registry,
            "mestier.service_rate.create",
            acme,
            &config(&[
                ("label", json!("Main-d’œuvre")),
                ("unit", json!("HOUR")),
                ("rate_cents", json!(6500)),
            ]),
        )
        .await,
    );
    let renamed = produced(
        run(
            &registry,
            "mestier.service_rate.update",
            acme,
            &config(&[
                ("service_rate_id", rate["id"].clone()),
                ("label", json!("Main-d’œuvre qualifiée")),
                ("unit", json!("HOUR")),
                ("rate_cents", json!(7000)),
            ]),
        )
        .await,
    );
    let equipment = produced(
        run(
            &registry,
            "mestier.equipment.create",
            acme,
            &config(&[
                ("name", json!("Nacelle")),
                ("hourly_rate_cents", json!(4500)),
            ]),
        )
        .await,
    );
    let unit = produced(
        run(
            &registry,
            "mestier.custom_unit.create",
            acme,
            &config(&[("code", json!("palette"))]),
        )
        .await,
    );
    let deleted = produced(
        run(
            &registry,
            "mestier.equipment.delete",
            acme,
            &config(&[("equipment_id", equipment["id"].clone())]),
        )
        .await,
    );

    assert_eq!(renamed["rate_cents"], 7000);
    assert_eq!(renamed["organization_id"], json!(acme.0.to_string()));
    same_keys(&rate, "mestier.service_rate.create");
    same_keys(&equipment, "mestier.equipment.create");
    same_keys(&unit, "mestier.custom_unit.create");
    assert_eq!(deleted["deleted"], json!(true));
    same_keys(&deleted, "mestier.equipment.delete");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn absences_and_employee_costs_follow_a_member_of_the_run_organization() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "ref-hr").await;
    let seat = member(&pool, acme).await;
    let registry = registry(&pool);

    let profile = produced(
        run(
            &registry,
            "mestier.employee_profile.upsert",
            acme,
            &config(&[
                ("member_id", json!(seat.to_string())),
                ("is_salaried", json!(false)),
                ("hourly_rate_cents", json!(2800)),
                ("weekly_contract_minutes", json!(2100)),
            ]),
        )
        .await,
    );
    let profiles = produced(
        run(
            &registry,
            "mestier.employee_profile.list",
            acme,
            &config(&[]),
        )
        .await,
    );
    let set = produced(
        run(
            &registry,
            "mestier.cost_basis.set",
            acme,
            &config(&[
                ("employee_id", profile["id"].clone()),
                ("effective_from", json!("2099-01-01")),
                ("is_salaried", json!(false)),
                ("hourly_rate_cents", json!(3000)),
                ("weekly_contract_minutes", json!(2100)),
            ]),
        )
        .await,
    );
    let history = produced(
        run(
            &registry,
            "mestier.cost_basis.list",
            acme,
            &config(&[("employee_id", profile["id"].clone())]),
        )
        .await,
    );
    let absence = produced(
        run(
            &registry,
            "mestier.absence.create",
            acme,
            &config(&[
                ("member_id", json!(seat.to_string())),
                ("kind", json!("LEAVE")),
                ("starts_at", json!("2026-07-01T00:00:00Z")),
                ("ends_at", json!("2026-07-02T00:00:00Z")),
                ("note", json!("Congés")),
            ]),
        )
        .await,
    );
    let cleared = produced(
        run(
            &registry,
            "mestier.absence.update",
            acme,
            &config(&[
                ("absence_id", absence["id"].clone()),
                ("clear_note", json!(true)),
            ]),
        )
        .await,
    );
    let removed = produced(
        run(
            &registry,
            "mestier.employee_profile.remove",
            acme,
            &config(&[("member_id", json!(seat.to_string()))]),
        )
        .await,
    );

    same_keys(&profile, "mestier.employee_profile.upsert");
    same_keys(&profiles, "mestier.employee_profile.list");
    same_keys(&set, "mestier.cost_basis.set");
    same_keys(&history, "mestier.cost_basis.list");
    assert_eq!(history.as_array().unwrap().len(), 2);
    same_keys(&absence, "mestier.absence.create");
    assert_eq!(cleared["note"], Value::Null);
    same_keys(&removed, "mestier.employee_profile.remove");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_object_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "ref-acme").await;
    let globex = world_support::organization(&pool, "ref-globex").await;
    let seat = member(&pool, acme).await;
    let registry = registry(&pool);

    let product = produced(
        run(
            &registry,
            "mestier.product.create",
            acme,
            &config(&[
                ("name", json!("Radiateur")),
                ("unit", json!("UNIT")),
                ("unit_price_cents", json!(24900)),
            ]),
        )
        .await,
    );
    let rate = produced(
        run(
            &registry,
            "mestier.service_rate.create",
            acme,
            &config(&[
                ("label", json!("Plomberie")),
                ("unit", json!("HOUR")),
                ("rate_cents", json!(6500)),
            ]),
        )
        .await,
    );
    let equipment = produced(
        run(
            &registry,
            "mestier.equipment.create",
            acme,
            &config(&[
                ("name", json!("Nacelle")),
                ("hourly_rate_cents", json!(4500)),
            ]),
        )
        .await,
    );
    let profile = produced(
        run(
            &registry,
            "mestier.employee_profile.upsert",
            acme,
            &config(&[
                ("member_id", json!(seat.to_string())),
                ("is_salaried", json!(false)),
                ("hourly_rate_cents", json!(2800)),
                ("weekly_contract_minutes", json!(2100)),
            ]),
        )
        .await,
    );
    let absence = produced(
        run(
            &registry,
            "mestier.absence.create",
            acme,
            &config(&[
                ("member_id", json!(seat.to_string())),
                ("kind", json!("SICK")),
                ("starts_at", json!("2026-07-01T00:00:00Z")),
                ("ends_at", json!("2026-07-02T00:00:00Z")),
            ]),
        )
        .await,
    );
    let history = produced(
        run(
            &registry,
            "mestier.cost_basis.list",
            acme,
            &config(&[("employee_id", profile["id"].clone())]),
        )
        .await,
    );
    let basis = history[0]["id"].clone();

    let cases = [
        (
            "mestier.product.get",
            vec![("product_id", product["id"].clone())],
        ),
        (
            "mestier.product.update",
            vec![
                ("product_id", product["id"].clone()),
                ("name", json!("Intrus")),
                ("unit", json!("UNIT")),
                ("unit_price_cents", json!(1)),
            ],
        ),
        (
            "mestier.product.delete",
            vec![("product_id", product["id"].clone())],
        ),
        (
            "mestier.service_rate.update",
            vec![
                ("service_rate_id", rate["id"].clone()),
                ("label", json!("Intrus")),
                ("unit", json!("HOUR")),
                ("rate_cents", json!(1)),
            ],
        ),
        (
            "mestier.service_rate.delete",
            vec![("service_rate_id", rate["id"].clone())],
        ),
        (
            "mestier.equipment.get",
            vec![("equipment_id", equipment["id"].clone())],
        ),
        (
            "mestier.equipment.delete",
            vec![("equipment_id", equipment["id"].clone())],
        ),
        (
            "mestier.absence.update",
            vec![
                ("absence_id", absence["id"].clone()),
                ("note", json!("Intrus")),
            ],
        ),
        (
            "mestier.absence.delete",
            vec![("absence_id", absence["id"].clone())],
        ),
        (
            "mestier.absence.create",
            vec![
                ("member_id", json!(seat.to_string())),
                ("kind", json!("LEAVE")),
                ("starts_at", json!("2026-08-01T00:00:00Z")),
                ("ends_at", json!("2026-08-02T00:00:00Z")),
            ],
        ),
        (
            "mestier.employee_profile.upsert",
            vec![
                ("member_id", json!(seat.to_string())),
                ("is_salaried", json!(false)),
                ("weekly_contract_minutes", json!(1)),
            ],
        ),
        (
            "mestier.employee_profile.remove",
            vec![("member_id", json!(seat.to_string()))],
        ),
        (
            "mestier.cost_basis.list",
            vec![("employee_id", profile["id"].clone())],
        ),
        (
            "mestier.cost_basis.set",
            vec![
                ("employee_id", profile["id"].clone()),
                ("effective_from", json!("2099-01-01")),
                ("is_salaried", json!(false)),
                ("weekly_contract_minutes", json!(1)),
            ],
        ),
        (
            "mestier.cost_basis.correct",
            vec![
                ("cost_basis_id", basis),
                ("effective_from", json!("2020-01-01")),
                ("is_salaried", json!(false)),
                ("weekly_contract_minutes", json!(1)),
            ],
        ),
    ];

    for (kind, pairs) in cases {
        let outcome = run(&registry, kind, globex, &config(&pairs)).await;

        assert_eq!(failure(outcome), NOT_FOUND, "{kind}");
    }

    let product_id: Uuid = product["id"].as_str().unwrap().parse().unwrap();
    let name: String = sqlx::query_scalar("SELECT name FROM products WHERE id = $1")
        .bind(product_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(name, "Radiateur");
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM absences WHERE member_id = $1",
            seat
        )
        .await,
        1
    );
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_dry_run_of_a_write_action_writes_nothing() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "ref-dry").await;
    let catalogue = connector_catalogue();
    let descriptor = catalogue.get("mestier.equipment.create", 1).unwrap();
    let config = config(&[
        ("name", json!("Nacelle")),
        ("hourly_rate_cents", json!(4500)),
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
        count(
            &pool,
            "SELECT count(*) FROM equipment WHERE org_id = $1",
            acme.0
        )
        .await,
        0
    );
}

#[tokio::test]
async fn arguments_are_validated_before_the_database() {
    let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused").unwrap();
    let registry = registry(&pool);
    let org = OrganizationId(Uuid::from_u128(1));
    let seat = json!(Uuid::from_u128(5).to_string());
    let cases = [
        (
            "mestier.absence.create",
            config(&[
                ("member_id", seat.clone()),
                ("kind", json!("HOLIDAY")),
                ("starts_at", json!("2026-07-01T00:00:00Z")),
                ("ends_at", json!("2026-07-02T00:00:00Z")),
            ]),
            "`kind` must be LEAVE, SICK or UNAVAILABLE",
        ),
        (
            "mestier.cost_basis.set",
            config(&[
                ("employee_id", seat.clone()),
                ("effective_from", json!("01/01/2026")),
                ("is_salaried", json!(false)),
                ("weekly_contract_minutes", json!(1)),
            ]),
            "`effective_from` must be a date formatted YYYY-MM-DD",
        ),
        (
            "mestier.equipment.create",
            config(&[("name", json!("x")), ("hourly_rate_cents", json!(-3))]),
            "`hourly_rate_cents` must be a whole number",
        ),
        (
            "mestier.employee_profile.upsert",
            config(&[("member_id", seat), ("weekly_contract_minutes", json!(1))]),
            "`is_salaried` must resolve to true or false",
        ),
    ];

    for (kind, config, expected) in cases {
        assert_eq!(
            failure(run(&registry, kind, org, &config).await),
            expected,
            "{kind}"
        );
    }
}
