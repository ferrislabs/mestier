use common::OrganizationId;
use serde_json::{Map, Value, json};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::application::MestierUseCase;
use crate::application::default_authorizer;
use crate::application::test_support::scratch_pool;
use crate::domain::automation::connector::{action_specs, connector_catalogue};
use crate::domain::automation::run::{ConnectorInput, ConnectorOutcome, DryRunInput};
use crate::infrastructure::automation::connectors::ConnectorRegistry;
use crate::infrastructure::automation::connectors::world_support::organization;
use crate::infrastructure::realtime::EventHub;

static SCRATCH: OnceCell<String> = OnceCell::const_new();

const FOREIGN: &str = "not found: the targeted object does not exist in this organization";

async fn pool() -> PgPool {
    scratch_pool("mestier_planning_b_test_", &SCRATCH).await
}

fn registry(pool: &PgPool) -> ConnectorRegistry {
    ConnectorRegistry::new(MestierUseCase::new(
        pool.clone(),
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

async fn count(pool: &PgPool, table: &str, organization_id: OrganizationId) -> i64 {
    sqlx::query_scalar(&format!("SELECT count(*) FROM {table} WHERE org_id = $1"))
        .bind(organization_id.0)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn every_planning_action_validates_before_touching_the_database() {
    let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused").unwrap();
    let registry = registry(&pool);
    let invalid = config(&[("limit", json!("not a number"))]);

    for spec in action_specs().filter(|spec| spec.group == "Planning") {
        let outcome = run(&registry, spec.kind, OrganizationId(Uuid::nil()), &invalid).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{}",
            spec.kind
        );
    }
}

#[tokio::test]
async fn invalid_arguments_name_the_field_without_echoing_it() {
    let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused").unwrap();
    let registry = registry(&pool);
    let secret = "do-not-leak-this-token";
    let org = OrganizationId(Uuid::nil());

    let cases = [
        (
            "mestier.task.get",
            vec![("task_id", json!(secret))],
            "`task_id` must be a UUID",
        ),
        (
            "mestier.task.create",
            vec![("title", json!("x"))],
            "`blocks_availability` must be true or false",
        ),
        (
            "mestier.planning.get_planning",
            vec![("from", json!("2026-01-01")), ("to", json!("2026-12-31"))],
            "planning window must not exceed 92 days",
        ),
        (
            "mestier.task.delete",
            vec![
                ("task_id", json!(Uuid::nil().to_string())),
                ("scope", json!(secret)),
            ],
            "`scope` must be one of this_occurrence, this_and_following",
        ),
        (
            "mestier.task_recurrence.create",
            vec![("frequency", json!("WEEKLY")), ("weekdays", json!([9]))],
            "`weekdays` must hold whole numbers from 1 to 7",
        ),
        (
            "mestier.project_template.replace_tasks",
            vec![
                ("template_id", json!(Uuid::nil().to_string())),
                ("tasks", json!([{ "title": "x" }])),
            ],
            "`tasks` element 0: `day_offset` is required",
        ),
    ];

    for (kind, pairs, expected) in cases {
        let error = failed(&registry, kind, org, &pairs).await;

        assert_eq!(error, expected, "{kind}");
        assert!(!error.contains(secret));
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_task_is_created_read_listed_updated_and_deleted_in_the_run_organization() {
    let pool = pool().await;
    let acme = organization(&pool, "acme").await;
    let registry = registry(&pool);

    let label = produced(
        &registry,
        "mestier.task_label.create",
        acme,
        &[("name", json!("Urgent")), ("color", json!("#E5484D"))],
    )
    .await;
    let created = produced(
        &registry,
        "mestier.task.create",
        acme,
        &[
            ("title", json!("Tailler la haie")),
            ("blocks_availability", json!(true)),
            ("starts_at", json!("2026-03-02T08:00:00Z")),
            ("ends_at", json!("2026-03-02T10:00:00Z")),
        ],
    )
    .await;
    let id = created["id"].clone();
    let got = produced(
        &registry,
        "mestier.task.get",
        acme,
        &[("task_id", id.clone())],
    )
    .await;
    let listed = produced(
        &registry,
        "mestier.task.list",
        acme,
        &[("statuses", json!(["PLANNED"]))],
    )
    .await;
    let patched = produced(
        &registry,
        "mestier.task.update",
        acme,
        &[
            ("task_id", id.clone()),
            ("title", json!("Tailler les haies")),
            ("label_ids", json!([label["id"].clone()])),
            ("description", Value::Null),
        ],
    )
    .await;
    let planning = produced(
        &registry,
        "mestier.planning.get_planning",
        acme,
        &[("from", json!("2026-03-01")), ("to", json!("2026-03-07"))],
    )
    .await;
    let deleted = produced(
        &registry,
        "mestier.task.delete",
        acme,
        &[("task_id", id.clone())],
    )
    .await;

    assert_same_keys(&created, "mestier.task.create");
    assert_same_keys(&got, "mestier.task.get");
    assert_same_keys(&listed, "mestier.task.list");
    assert_same_keys(&patched, "mestier.task.update");
    assert_same_keys(&planning, "mestier.planning.get_planning");
    assert_eq!(got["id"], id);
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(patched["task"]["title"], "Tailler les haies");
    assert_eq!(patched["task"]["labels"].as_array().unwrap().len(), 1);
    assert_eq!(patched["task"]["description"], Value::Null);
    assert_eq!(planning["entries"].as_array().unwrap().len(), 1);
    assert_eq!(deleted["deleted"], true);
    assert_eq!(
        failed(&registry, "mestier.task.get", acme, &[("task_id", id)]).await,
        FOREIGN
    );
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_task_object_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let pool = pool().await;
    let acme = organization(&pool, "acme").await;
    let globex = organization(&pool, "globex").await;
    let registry = registry(&pool);
    let task = produced(
        &registry,
        "mestier.task.create",
        acme,
        &[
            ("title", json!("Chez Acme")),
            ("blocks_availability", json!(true)),
        ],
    )
    .await;
    let target = task["id"].clone();

    for (kind, extra) in [
        ("mestier.task.get", vec![]),
        ("mestier.task.update", vec![("title", json!("Intrus"))]),
        ("mestier.task.delete", vec![]),
        ("mestier.task_comment.list", vec![]),
    ] {
        let mut pairs = vec![("task_id", target.clone())];
        pairs.extend(extra);

        assert_eq!(
            failed(&registry, kind, globex, &pairs).await,
            FOREIGN,
            "{kind}"
        );
    }

    let still = produced(&registry, "mestier.task.get", acme, &[("task_id", target)]).await;
    assert_eq!(still["title"], "Chez Acme");
    assert_eq!(
        produced(&registry, "mestier.task.list", globex, &[]).await,
        json!([])
    );
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn projects_templates_labels_and_recurrences_are_managed_and_stay_in_their_organization() {
    let pool = pool().await;
    let acme = organization(&pool, "acme").await;
    let globex = organization(&pool, "globex").await;
    let registry = registry(&pool);

    let project = produced(
        &registry,
        "mestier.project.create_project",
        acme,
        &[("name", json!("Jardin Fontaine"))],
    )
    .await;
    let template = produced(
        &registry,
        "mestier.project_template.create",
        acme,
        &[
            ("name", json!("Création de jardin")),
            (
                "tasks",
                json!([{ "title": "Préparer", "day_offset": 0, "all_day": true, "blocks_availability": true }]),
            ),
        ],
    )
    .await;
    let instantiated = produced(
        &registry,
        "mestier.project_template.instantiate",
        acme,
        &[
            ("template_id", template["id"].clone()),
            ("name", json!("Jardin Dupont")),
            ("start_date", json!("2026-03-02")),
        ],
    )
    .await;
    let label = produced(
        &registry,
        "mestier.task_label.create",
        acme,
        &[("name", json!("Urgent")), ("color", json!("#E5484D"))],
    )
    .await;
    let recurrence = produced(
        &registry,
        "mestier.task_recurrence.create",
        acme,
        &[
            ("frequency", json!("WEEKLY")),
            ("weekdays", json!([2])),
            ("starts_on", json!("2026-03-03")),
            ("timezone", json!("Europe/Paris")),
            ("start_time", json!("09:00")),
            ("duration_minutes", json!(60)),
            ("title", json!("Réunion")),
            ("blocks_availability", json!(true)),
        ],
    )
    .await;

    assert_same_keys(&project, "mestier.project.create_project");
    assert_same_keys(&template, "mestier.project_template.create");
    assert_same_keys(&instantiated, "mestier.project_template.instantiate");
    assert_same_keys(&label, "mestier.task_label.create");
    assert_same_keys(&recurrence, "mestier.task_recurrence.create");
    assert_eq!(instantiated["tasks"].as_array().unwrap().len(), 1);

    for (kind, pairs) in [
        (
            "mestier.project.get",
            vec![("project_id", project["id"].clone())],
        ),
        (
            "mestier.project.archive",
            vec![("project_id", project["id"].clone())],
        ),
        (
            "mestier.project.create_channel",
            vec![("project_id", project["id"].clone())],
        ),
        (
            "mestier.project_template.get",
            vec![("template_id", template["id"].clone())],
        ),
        (
            "mestier.project_template.archive",
            vec![("template_id", template["id"].clone())],
        ),
        (
            "mestier.task_label.delete",
            vec![("label_id", label["id"].clone())],
        ),
        (
            "mestier.task_recurrence.delete",
            vec![("recurrence_id", recurrence["id"].clone())],
        ),
    ] {
        assert_eq!(
            failed(&registry, kind, globex, &pairs).await,
            FOREIGN,
            "{kind}"
        );
    }

    let kept = produced(
        &registry,
        "mestier.project.get",
        acme,
        &[("project_id", project["id"].clone())],
    )
    .await;
    assert_eq!(kept["archived_at"], Value::Null);
    assert_eq!(
        produced(&registry, "mestier.task_label.list", globex, &[]).await,
        json!([])
    );
    assert_eq!(
        produced(&registry, "mestier.task_recurrence.list", globex, &[]).await,
        json!([])
    );
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_dry_run_returns_the_sample_and_writes_nothing() {
    let pool = pool().await;
    let acme = organization(&pool, "acme").await;
    let descriptor = connector_catalogue()
        .get("mestier.task.create", 1)
        .unwrap()
        .clone();
    let pairs = config(&[
        ("title", json!("Fantôme")),
        ("blocks_availability", json!(true)),
    ]);

    let outcome = ConnectorRegistry::dry_run(
        &descriptor,
        &DryRunInput {
            config: &pairs,
            stored: None,
        },
    );

    assert_eq!(
        outcome,
        Some(ConnectorOutcome::Produced(descriptor.sample_output.clone()))
    );
    assert_eq!(count(&pool, "tasks", acme).await, 0);
    assert_eq!(count(&pool, "task_labels", acme).await, 0);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_comment_is_written_by_the_system_user_and_only_that_user_may_change_it() {
    let pool = pool().await;
    let acme = organization(&pool, "acme").await;
    let registry = registry(&pool);
    let task = produced(
        &registry,
        "mestier.task.create",
        acme,
        &[
            ("title", json!("Tailler la haie")),
            ("blocks_availability", json!(false)),
        ],
    )
    .await;

    let created = produced(
        &registry,
        "mestier.task_comment.create",
        acme,
        &[
            ("task_id", task["id"].clone()),
            ("body", json!("Matériel livré.")),
        ],
    )
    .await;
    let updated = produced(
        &registry,
        "mestier.task_comment.update",
        acme,
        &[
            ("comment_id", created["id"].clone()),
            ("body", json!("Matériel livré et vérifié.")),
        ],
    )
    .await;
    let listed = produced(
        &registry,
        "mestier.task_comment.list",
        acme,
        &[("task_id", task["id"].clone())],
    )
    .await;

    assert_same_keys(&created, "mestier.task_comment.create");
    assert_eq!(created["author"]["display_name"], "Automatisation");
    assert_eq!(updated["body"], "Matériel livré et vérifié.");
    assert_eq!(listed[0]["author"]["display_name"], "Automatisation");
    assert_eq!(listed[0]["author"]["id"], created["author"]["id"]);

    let human: Uuid = sqlx::query_scalar("SELECT owner_id FROM organizations WHERE id = $1")
        .bind(acme.0)
        .fetch_one(&pool)
        .await
        .unwrap();
    let foreign_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO task_comments (id, org_id, task_id, author_user_id, body) \
         VALUES ($1, $2, $3, $4, 'Humain')",
    )
    .bind(foreign_id)
    .bind(acme.0)
    .bind(task["id"].as_str().unwrap().parse::<Uuid>().unwrap())
    .bind(human)
    .execute(&pool)
    .await
    .unwrap();

    for kind in ["mestier.task_comment.update", "mestier.task_comment.delete"] {
        let error = failed(
            &registry,
            kind,
            acme,
            &[("comment_id", json!(foreign_id)), ("body", json!("Pris"))],
        )
        .await;
        assert!(error.starts_with("forbidden"), "{kind}: {error}");
    }

    let deleted = produced(
        &registry,
        "mestier.task_comment.delete",
        acme,
        &[("comment_id", created["id"].clone())],
    )
    .await;
    assert_eq!(deleted["deleted"], true);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_comment_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let pool = pool().await;
    let acme = organization(&pool, "acme").await;
    let globex = organization(&pool, "globex").await;
    let registry = registry(&pool);
    let task = produced(
        &registry,
        "mestier.task.create",
        acme,
        &[
            ("title", json!("Chez Acme")),
            ("blocks_availability", json!(false)),
        ],
    )
    .await;
    let comment = produced(
        &registry,
        "mestier.task_comment.create",
        acme,
        &[("task_id", task["id"].clone()), ("body", json!("Privé"))],
    )
    .await;

    assert_eq!(
        failed(
            &registry,
            "mestier.task_comment.create",
            globex,
            &[("task_id", task["id"].clone()), ("body", json!("Intrus"))],
        )
        .await,
        FOREIGN
    );
    for kind in ["mestier.task_comment.update", "mestier.task_comment.delete"] {
        assert_eq!(
            failed(
                &registry,
                kind,
                globex,
                &[
                    ("comment_id", comment["id"].clone()),
                    ("body", json!("Intrus"))
                ],
            )
            .await,
            FOREIGN,
            "{kind}"
        );
    }

    let listed = produced(
        &registry,
        "mestier.task_comment.list",
        acme,
        &[("task_id", task["id"].clone())],
    )
    .await;
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["body"], "Privé");
}
