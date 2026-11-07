use common::OrganizationId;
use serde_json::{Map, Value, json};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::application::maintenance_intervention::tests::{World, rolling};
use crate::application::test_support::{purge, scratch_pool};
use crate::domain::automation::connector::connector_catalogue;
use crate::domain::automation::run::{ConnectorInput, ConnectorOutcome, DryRunInput};
use crate::infrastructure::automation::connectors::ConnectorRegistry;
use crate::{InterventionStatus, StarterModel};

static SCRATCH: OnceCell<String> = OnceCell::const_new();

const NOT_FOUND: &str = "not found: the targeted object does not exist in this organization";

async fn pool() -> PgPool {
    scratch_pool("mestier_actions_planning_a_", &SCRATCH).await
}

fn config(pairs: &[(&str, Value)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

async fn run(
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

async fn produced(
    registry: &ConnectorRegistry,
    kind: &str,
    organization_id: OrganizationId,
    pairs: &[(&str, Value)],
) -> Value {
    match run(registry, kind, organization_id, &config(pairs)).await {
        ConnectorOutcome::Produced(value) => value,
        other => panic!("{kind}: expected an output, got {other:?}"),
    }
}

async fn failed(
    registry: &ConnectorRegistry,
    kind: &str,
    organization_id: OrganizationId,
    pairs: &[(&str, Value)],
) -> String {
    match run(registry, kind, organization_id, &config(pairs)).await {
        ConnectorOutcome::Failed { error } => error,
        other => panic!("{kind}: expected a failure, got {other:?}"),
    }
}

fn assert_same_shape(actual: &Value, kind: &str) {
    let sample = connector_catalogue()
        .get(kind, 1)
        .unwrap()
        .sample_output
        .clone();
    let first = |value: &Value| value.get(0).cloned().unwrap_or_else(|| value.clone());
    let keys = |value: Value| {
        let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    };
    assert_eq!(keys(first(actual)), keys(first(&sample)), "{kind}");
}

fn sections() -> Value {
    serde_json::to_value(StarterModel::Landscaping.sections()).unwrap()
}

fn text(value: &Value, key: &str) -> Value {
    json!(value[key].as_str().unwrap())
}

async fn cleanup(world: World) {
    for statement in [
        "DELETE FROM does WHERE org_id = $1",
        "DELETE FROM doe_models WHERE org_id = $1",
        "DELETE FROM doe_settings WHERE org_id = $1",
        "DELETE FROM work_slots WHERE org_id = $1",
        "DELETE FROM employee_rhythm_slots WHERE rhythm_id IN (SELECT id FROM employee_rhythms WHERE org_id = $1)",
        "DELETE FROM employee_rhythms WHERE org_id = $1",
    ] {
        purge(&world.pool, statement, world.organization_id.0).await;
    }
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_model_is_created_listed_read_archived_and_restored_in_the_run_organization() {
    let world = World::new(&pool().await).await;
    let registry = ConnectorRegistry::new(world.usecase.clone());
    let org = world.organization_id;

    let created = produced(
        &registry,
        "mestier.doe_model.create",
        org,
        &[("name", json!("Paysagiste")), ("sections", sections())],
    )
    .await;
    let target = text(&created, "id");
    let listed = produced(&registry, "mestier.doe_model.list", org, &[]).await;
    let got = produced(
        &registry,
        "mestier.doe_model.get",
        org,
        &[("doe_model_id", target.clone())],
    )
    .await;
    let archived = produced(
        &registry,
        "mestier.doe_model.archive",
        org,
        &[("doe_model_id", target.clone())],
    )
    .await;
    let restored = produced(
        &registry,
        "mestier.doe_model.restore",
        org,
        &[("doe_model_id", target)],
    )
    .await;

    assert_same_shape(&created, "mestier.doe_model.create");
    assert_same_shape(&listed, "mestier.doe_model.list");
    assert_same_shape(&got, "mestier.doe_model.get");
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert!(archived["archived_at"].is_string());
    assert!(restored["archived_at"].is_null());

    cleanup(world).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_contract_is_read_listed_and_its_grid_names_what_a_run_returns() {
    let world = World::new(&pool().await).await;
    let registry = ConnectorRegistry::new(world.usecase.clone());
    let org = world.organization_id;
    let site = world.seed_site("Mairie", Some("Lyon")).await;
    let contract = world
        .seed_contract(&site, (2026, 1, 52), &[("Tonte", rolling(14))])
        .await;
    let id = json!(contract.id.to_string());

    let got = produced(
        &registry,
        "mestier.maintenance_contract.get",
        org,
        &[("contract_id", id.clone())],
    )
    .await;
    let listed = produced(
        &registry,
        "mestier.maintenance_contract.list",
        org,
        &[("season", json!(2026))],
    )
    .await;
    let grid = produced(
        &registry,
        "mestier.maintenance.get_grid",
        org,
        &[("season", json!(2026))],
    )
    .await;

    assert_same_shape(&got, "mestier.maintenance_contract.get");
    assert_same_shape(&listed, "mestier.maintenance_contract.list");
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(grid["contracts"].as_array().unwrap().len(), 1);
    assert_eq!(grid["contracts"][0]["lines"][0]["label"], "Tonte");

    cleanup(world).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_proposed_visit_is_validated_by_the_named_member_and_a_stranger_is_refused() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let registry = ConnectorRegistry::new(acme.usecase.clone());
    let site = acme.seed_site("Mairie", None).await;
    let contract = acme
        .seed_contract(&site, (2026, 1, 52), &[("Tonte", rolling(14))])
        .await;
    let visit = acme
        .seed_visit(
            contract.line_ids[0],
            chrono::NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
            InterventionStatus::Proposed,
        )
        .await;
    let visit = json!(visit.0.to_string());

    let proposed = produced(
        &registry,
        "mestier.maintenance_intervention.list",
        acme.organization_id,
        &[],
    )
    .await;
    let stranger = failed(
        &registry,
        "mestier.maintenance_intervention.validate",
        acme.organization_id,
        &[
            ("intervention_id", visit.clone()),
            ("member_id", json!(globex.manager.member_id.0.to_string())),
        ],
    )
    .await;
    let validated = produced(
        &registry,
        "mestier.maintenance_intervention.validate",
        acme.organization_id,
        &[
            ("intervention_id", visit),
            ("member_id", json!(acme.manager.member_id.0.to_string())),
        ],
    )
    .await;

    assert_same_shape(&proposed, "mestier.maintenance_intervention.list");
    assert_eq!(proposed.as_array().unwrap().len(), 1);
    assert_eq!(stranger, NOT_FOUND);
    assert_eq!(validated["status"], "VALIDATED");
    assert_eq!(
        validated["resolved_by"],
        json!(acme.manager.member_id.0.to_string())
    );
    assert_same_shape(&validated, "mestier.maintenance_intervention.validate");

    cleanup(acme).await;
    cleanup(globex).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn work_slots_are_replaced_then_read_back() {
    let world = World::new(&pool().await).await;
    let registry = ConnectorRegistry::new(world.usecase.clone());
    let org = world.organization_id;
    let member = json!(world.plain.member_id.0.to_string());

    let written = produced(
        &registry,
        "mestier.work_time.put_work_slots",
        org,
        &[
            ("member_id", member.clone()),
            ("from", json!("2026-01-05")),
            ("to", json!("2026-01-11")),
            (
                "slots",
                json!([{ "work_date": "2026-01-05", "starts_minute": 480, "ends_minute": 720 }]),
            ),
        ],
    )
    .await;
    let read = produced(
        &registry,
        "mestier.work_time.get",
        org,
        &[
            ("member_id", member),
            ("from", json!("2026-01-05")),
            ("to", json!("2026-01-11")),
        ],
    )
    .await;

    assert_same_shape(&written, "mestier.work_time.put_work_slots");
    assert_eq!(read["work_slots"].as_array().unwrap().len(), 1);
    assert_eq!(read["work_slots"][0]["ends_minute"], 720);

    cleanup(world).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_object_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let registry = ConnectorRegistry::new(acme.usecase.clone());
    let org = globex.organization_id;
    let model = produced(
        &registry,
        "mestier.doe_model.create",
        acme.organization_id,
        &[("name", json!("Chez Acme")), ("sections", sections())],
    )
    .await;
    let site = acme.seed_site("Mairie", None).await;
    let contract = acme
        .seed_contract(&site, (2026, 1, 52), &[("Tonte", rolling(14))])
        .await;
    let model_id = text(&model, "id");
    let contract_id = json!(contract.id.to_string());
    let member = json!(acme.plain.member_id.0.to_string());

    let cases: Vec<(&str, Vec<(&str, Value)>)> = vec![
        (
            "mestier.doe_model.get",
            vec![("doe_model_id", model_id.clone())],
        ),
        (
            "mestier.doe_model.update",
            vec![
                ("doe_model_id", model_id.clone()),
                ("name", json!("Intrus")),
                ("sections", sections()),
            ],
        ),
        (
            "mestier.doe_model.archive",
            vec![("doe_model_id", model_id.clone())],
        ),
        (
            "mestier.doe_model.restore",
            vec![("doe_model_id", model_id.clone())],
        ),
        (
            "mestier.maintenance_contract.get",
            vec![("contract_id", contract_id.clone())],
        ),
        (
            "mestier.maintenance_contract.archive",
            vec![("contract_id", contract_id.clone())],
        ),
        (
            "mestier.maintenance_contract.duplicate",
            vec![("contract_id", contract_id.clone())],
        ),
        (
            "mestier.work_time.get",
            vec![
                ("member_id", member.clone()),
                ("from", json!("2026-01-05")),
                ("to", json!("2026-01-11")),
            ],
        ),
        (
            "mestier.work_time.put_work_slots",
            vec![
                ("member_id", member.clone()),
                ("from", json!("2026-01-05")),
                ("to", json!("2026-01-11")),
                ("slots", json!([])),
            ],
        ),
        (
            "mestier.work_time.put_rhythm",
            vec![
                ("member_id", member),
                ("effective_from", json!("2026-01-05")),
                ("slots", json!([])),
            ],
        ),
    ];
    for (kind, pairs) in cases {
        assert_eq!(
            failed(&registry, kind, org, &pairs).await,
            NOT_FOUND,
            "{kind}"
        );
    }

    let still = produced(
        &registry,
        "mestier.doe_model.get",
        acme.organization_id,
        &[("doe_model_id", model_id)],
    )
    .await;
    let kept = produced(
        &registry,
        "mestier.maintenance_contract.get",
        acme.organization_id,
        &[("contract_id", contract_id)],
    )
    .await;
    assert_eq!(still["name"], "Chez Acme");
    assert!(still["archived_at"].is_null());
    assert!(kept["archived_at"].is_null());

    cleanup(acme).await;
    cleanup(globex).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_dry_run_of_a_write_action_writes_nothing() {
    let world = World::new(&pool().await).await;
    let registry = ConnectorRegistry::new(world.usecase.clone());
    let model = produced(
        &registry,
        "mestier.doe_model.create",
        world.organization_id,
        &[("name", json!("Intact")), ("sections", sections())],
    )
    .await;
    let catalogue = connector_catalogue();
    let descriptor = catalogue.get("mestier.doe_model.archive", 1).unwrap();
    let dry = config(&[("doe_model_id", text(&model, "id"))]);

    let outcome = ConnectorRegistry::dry_run(
        descriptor,
        &DryRunInput {
            config: &dry,
            stored: None,
        },
    );
    let after = produced(
        &registry,
        "mestier.doe_model.get",
        world.organization_id,
        &[("doe_model_id", text(&model, "id"))],
    )
    .await;

    assert_eq!(
        outcome,
        Some(ConnectorOutcome::Produced(descriptor.sample_output.clone()))
    );
    assert!(after["archived_at"].is_null());

    cleanup(world).await;
}
