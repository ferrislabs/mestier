//! WS2 of the Entretien chantier: creating, reading, replacing, archiving and
//! duplicating maintenance contracts. Scenarios `@spec-ent-9`, `@spec-ent-10`
//! and `@spec-ent-11` of `docs/chantiers/entretien.md`, plus the refusals the
//! issue asks for.

use mestier_core::Permissions;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{harness, issuer};

struct Intruder {
    token: String,
    organization_id: Uuid,
    user_id: Uuid,
}

impl Intruder {
    async fn cleanup(&self, pool: &PgPool) {
        for statement in [
            "DELETE FROM member_roles WHERE role_id IN (SELECT id FROM roles WHERE organization_id = $1)",
            "DELETE FROM roles WHERE organization_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            sqlx::query(statement)
                .bind(self.organization_id)
                .execute(pool)
                .await
                .unwrap_or_else(|e| panic!("intruder cleanup failed on `{statement}`: {e}"));
        }
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(self.user_id)
            .execute(pool)
            .await
            .expect("clear the intruder user");
    }
}

/// A member of another organization who holds `planning.manage` there, so a
/// 404 below can only come from the contract belonging elsewhere.
async fn seed_intruder(pool: &PgPool) -> Intruder {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-maintenance-intruder-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("maintenance-intruder-{user_id}@example.com"))
    .bind(format!("maintenance-intruder-{user_id}"))
    .bind("Intruder")
    .bind(&sub)
    .execute(pool)
    .await
    .expect("seed the intruder user");

    let organization_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id)
        .bind("Intruder Org")
        .bind(format!("maintenance-intruder-org-{organization_id}"))
        .bind(user_id)
        .execute(pool)
        .await
        .expect("seed the intruder's organization");

    let member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(organization_id)
    .bind(user_id)
    .bind("Intruder")
    .execute(pool)
    .await
    .expect("seed the intruder's membership");

    let role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, 'all', $3)",
    )
    .bind(role_id)
    .bind(organization_id)
    .bind(Permissions::ALL.0)
    .execute(pool)
    .await
    .expect("seed the intruder's role");
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(member_id)
        .bind(role_id)
        .execute(pool)
        .await
        .expect("assign the intruder's role");

    Intruder {
        token: issuer::mint(&sub),
        organization_id,
        user_id,
    }
}

pub(crate) struct Site {
    pub(crate) customer_id: Uuid,
    pub(crate) context_id: Uuid,
}

async fn seed_site(app: &harness::App, customer: &str, label: &str) -> Site {
    seed_site_in(app, customer, label, "Vichy").await
}

pub(crate) async fn seed_site_in(
    app: &harness::App,
    customer: &str,
    label: &str,
    city: &str,
) -> Site {
    let customer_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(customer_id)
        .bind(app.organization_id)
        .bind(customer)
        .execute(&app.pool)
        .await
        .expect("seed the customer");

    let context_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO customer_contexts (id, customer_id, label, city) VALUES ($1, $2, $3, $4)",
    )
    .bind(context_id)
    .bind(customer_id)
    .bind(label)
    .bind(city)
    .execute(&app.pool)
    .await
    .expect("seed the site");

    Site {
        customer_id,
        context_id,
    }
}

pub(crate) async fn cleanup(app: &harness::App) {
    for statement in [
        "DELETE FROM maintenance_interventions WHERE org_id = $1",
        "DELETE FROM maintenance_lines WHERE org_id = $1",
        "DELETE FROM maintenance_contracts WHERE org_id = $1",
        "DELETE FROM projects WHERE org_id = $1",
        "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
        "DELETE FROM customers WHERE org_id = $1",
    ] {
        sqlx::query(statement)
            .bind(app.organization_id)
            .execute(&app.pool)
            .await
            .unwrap_or_else(|e| panic!("maintenance cleanup failed on `{statement}`: {e}"));
    }
    app.cleanup().await;
}

fn contracts_url(app: &harness::App, organization_id: Uuid, suffix: &str) -> String {
    format!(
        "{}/api/v1/organizations/{organization_id}/maintenance/contracts{suffix}",
        app.base_url
    )
}

fn tonte() -> Value {
    json!({
        "label": "Tonte avec ramassage",
        "cadence": { "kind": "ROLLING", "every_days": 14 },
        "minutes_per_intervention": 90
    })
}

fn taille() -> Value {
    json!({
        "label": "Taille",
        "cadence": { "kind": "WINDOWS", "windows": [
            { "from_week": 14, "to_week": 17 },
            { "from_week": 44, "to_week": 47 }
        ] }
    })
}

fn create_body(site: &Site, lines: Vec<Value>) -> Value {
    json!({
        "customer_id": site.customer_id,
        "customer_context_id": site.context_id,
        "reference": "C25029",
        "season": { "year": 2026, "first_week": 12, "last_week": 48 },
        "amount_cents": 250000,
        "lines": lines
    })
}

pub(crate) async fn create_contract(app: &harness::App, site: &Site, lines: Vec<Value>) -> Value {
    let created = reqwest::Client::new()
        .post(contracts_url(app, app.organization_id, ""))
        .bearer_auth(&app.token)
        .json(&create_body(site, lines))
        .send()
        .await
        .expect("the api answers the create call");
    let status = created.status();
    let body: Value = created.json().await.expect("the create answer is json");
    assert_eq!(status, 201, "{body}");
    body
}

/// What the form sends back: the contract as read, without what is fixed.
fn put_body(contract: &Value) -> Value {
    let data = &contract["data"];
    json!({
        "reference": data["reference"],
        "season": data["season"],
        "amount_cents": data["amount_cents"],
        "lines": data["lines"],
    })
}

async fn manager_member_id(app: &harness::App) -> Uuid {
    sqlx::query_scalar(
        "SELECT id FROM organization_members WHERE organization_id = $1 AND last_name = 'Manager'",
    )
    .bind(app.organization_id)
    .fetch_one(&app.pool)
    .await
    .expect("read the manager's seat")
}

async fn seed_interventions(app: &harness::App, line_id: &str, count: u32) {
    let line_id: Uuid = line_id.parse().expect("a line id");
    let member_id = manager_member_id(app).await;
    for day in 0..count {
        sqlx::query(
            "INSERT INTO maintenance_interventions \
             (id, org_id, line_id, done_on, status, proposed_by, resolved_by, resolved_at) \
             VALUES ($1, $2, $3, DATE '2026-04-01' + $4::int * 14, 'VALIDATED', $5, $5, now())",
        )
        .bind(Uuid::now_v7())
        .bind(app.organization_id)
        .bind(line_id)
        .bind(day as i32)
        .bind(member_id)
        .execute(&app.pool)
        .await
        .expect("seed the intervention");
    }
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_contract_is_created_with_its_project_read_listed_replaced_and_archived() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let site = seed_site(&app, "GAUME", "Parc").await;

    let created = create_contract(&app, &site, vec![tonte(), taille()]).await;
    let contract_id = created["data"]["id"].as_str().unwrap().to_owned();
    assert_eq!(created["data"]["status"], "ACTIVE");
    assert_eq!(created["data"]["lines"].as_array().unwrap().len(), 2);
    assert_eq!(created["data"]["lines"][0]["label"], "Tonte avec ramassage");
    assert_eq!(created["data"]["lines"][1]["cadence"]["kind"], "WINDOWS");

    let project_name: String =
        sqlx::query_scalar("SELECT name FROM projects WHERE id = $1 AND org_id = $2")
            .bind(
                created["data"]["project_id"]
                    .as_str()
                    .unwrap()
                    .parse::<Uuid>()
                    .unwrap(),
            )
            .bind(app.organization_id)
            .fetch_one(&app.pool)
            .await
            .expect("the contract's project exists");
    assert_eq!(project_name, "Entretien 2026 - GAUME - Parc");

    let read: Value = client
        .get(contracts_url(
            &app,
            app.organization_id,
            &format!("/{contract_id}"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("read")
        .json()
        .await
        .expect("json");
    assert_eq!(read["data"], created["data"]);

    let listed: Value = client
        .get(contracts_url(&app, app.organization_id, "?season=2026"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    let other_season: Value = client
        .get(contracts_url(&app, app.organization_id, "?season=2027"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    assert!(other_season["data"].as_array().unwrap().is_empty());

    let mut body = put_body(&created);
    body["reference"] = json!("C25030");
    body["lines"] = json!([created["data"]["lines"][0].clone(), {
        "label": "Arrosage",
        "cadence": { "kind": "ON_DEMAND" }
    }]);
    let replaced = client
        .put(contracts_url(
            &app,
            app.organization_id,
            &format!("/{contract_id}"),
        ))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("replace");
    assert_eq!(replaced.status(), 200);
    let replaced: Value = replaced.json().await.expect("json");
    assert_eq!(replaced["data"]["reference"], "C25030");
    assert_eq!(
        replaced["data"]["lines"][0]["id"],
        created["data"]["lines"][0]["id"]
    );
    assert_eq!(replaced["data"]["lines"][1]["cadence"]["kind"], "ON_DEMAND");
    assert_eq!(
        replaced["data"]["project_id"],
        created["data"]["project_id"]
    );

    let archived = client
        .post(contracts_url(
            &app,
            app.organization_id,
            &format!("/{contract_id}/archive"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("archive");
    assert_eq!(archived.status(), 200);
    let archived: Value = archived.json().await.expect("json");
    assert!(archived["data"]["archived_at"].is_string());

    let hidden: Value = client
        .get(contracts_url(&app, app.organization_id, "?season=2026"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    assert!(hidden["data"].as_array().unwrap().is_empty());
    let shown: Value = client
        .get(contracts_url(
            &app,
            app.organization_id,
            "?season=2026&include_archived=true",
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    assert_eq!(shown["data"].as_array().unwrap().len(), 1);

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn invalid_seasons_cadences_and_references_are_refused_with_409_or_404() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let site = seed_site(&app, "GAUME", "Parc").await;
    let other_site = seed_site(&app, "AUTRE", "Cour").await;

    let post = |body: Value| {
        client
            .post(contracts_url(&app, app.organization_id, ""))
            .bearer_auth(&app.token)
            .json(&body)
            .send()
    };

    let outside = post(create_body(
        &site,
        vec![json!({
            "label": "Taille",
            "cadence": { "kind": "WINDOWS", "windows": [{ "from_week": 46, "to_week": 50 }] }
        })],
    ))
    .await
    .expect("outside");
    assert_eq!(outside.status(), 409, "a window outside the season");

    let zero_interval = post(create_body(
        &site,
        vec![json!({ "label": "Tonte", "cadence": { "kind": "ROLLING", "every_days": 0 } })],
    ))
    .await
    .expect("zero");
    assert_eq!(zero_interval.status(), 409, "an interval of zero days");

    let mut bad_season = create_body(&site, vec![tonte()]);
    bad_season["season"] = json!({ "year": 2027, "first_week": 12, "last_week": 53 });
    assert_eq!(post(bad_season).await.expect("season").status(), 409);

    let mut foreign_site = create_body(&site, vec![tonte()]);
    foreign_site["customer_context_id"] = json!(other_site.context_id);
    assert_eq!(
        post(foreign_site).await.expect("site").status(),
        409,
        "a site that is not the customer's"
    );

    let mut unknown_customer = create_body(&site, vec![tonte()]);
    unknown_customer["customer_id"] = json!(Uuid::now_v7());
    assert_eq!(
        post(unknown_customer).await.expect("customer").status(),
        404
    );

    let mut unknown_manager = create_body(&site, vec![tonte()]);
    unknown_manager["manager_member_id"] = json!(Uuid::now_v7());
    assert_eq!(post(unknown_manager).await.expect("manager").status(), 404);

    let none: Value = client
        .get(contracts_url(&app, app.organization_id, "?season=2026"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    assert!(
        none["data"].as_array().unwrap().is_empty(),
        "nothing was left behind"
    );

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn replacing_the_lines_refuses_to_drop_one_with_interventions_and_allows_one_without() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let site = seed_site(&app, "GAUME", "Parc").await;
    let created = create_contract(&app, &site, vec![tonte(), taille()]).await;
    let contract_id = created["data"]["id"].as_str().unwrap().to_owned();
    let tonte_id = created["data"]["lines"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    seed_interventions(&app, &tonte_id, 3).await;
    let url = contracts_url(&app, app.organization_id, &format!("/{contract_id}"));

    let mut without_tonte = put_body(&created);
    without_tonte["lines"] = json!([created["data"]["lines"][1].clone()]);
    let refused = client
        .put(&url)
        .bearer_auth(&app.token)
        .json(&without_tonte)
        .send()
        .await
        .expect("refused");
    assert_eq!(refused.status(), 409);

    let intact: Value = client
        .get(&url)
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("read")
        .json()
        .await
        .expect("json");
    assert_eq!(
        intact["data"]["lines"].as_array().unwrap().len(),
        2,
        "nothing was dropped"
    );

    let mut without_taille = put_body(&created);
    without_taille["lines"] = json!([created["data"]["lines"][0].clone()]);
    let allowed = client
        .put(&url)
        .bearer_auth(&app.token)
        .json(&without_taille)
        .send()
        .await
        .expect("allowed");
    assert_eq!(allowed.status(), 200);
    let allowed: Value = allowed.json().await.expect("json");
    assert_eq!(allowed["data"]["lines"].as_array().unwrap().len(), 1);

    let mut foreign_line = put_body(&created);
    foreign_line["lines"] = json!([{
        "id": Uuid::now_v7(),
        "label": "Tonte",
        "cadence": { "kind": "ON_DEMAND" }
    }]);
    let foreign = client
        .put(&url)
        .bearer_auth(&app.token)
        .json(&foreign_line)
        .send()
        .await
        .expect("foreign");
    assert_eq!(foreign.status(), 409);

    let mut other_year = put_body(&created);
    other_year["season"] = json!({ "year": 2027, "first_week": 12, "last_week": 48 });
    let other_year = client
        .put(&url)
        .bearer_auth(&app.token)
        .json(&other_year)
        .send()
        .await
        .expect("year");
    assert_eq!(other_year.status(), 409);

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_ent_9_another_organizations_member_gets_404_on_read_write_archive_and_duplicate() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let intruder = seed_intruder(&app.pool).await;
    let site = seed_site(&app, "GAUME", "Parc").await;
    let created = create_contract(&app, &site, vec![tonte()]).await;
    let contract_id = created["data"]["id"].as_str().unwrap().to_owned();
    let url = |suffix: &str| {
        contracts_url(
            &app,
            intruder.organization_id,
            &format!("/{contract_id}{suffix}"),
        )
    };

    let read = client
        .get(url(""))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("read");
    assert_eq!(read.status(), 404);

    let write = client
        .put(url(""))
        .bearer_auth(&intruder.token)
        .json(&put_body(&created))
        .send()
        .await
        .expect("write");
    assert_eq!(write.status(), 404);

    for suffix in ["/archive", "/duplicate"] {
        let attempt = client
            .post(url(suffix))
            .bearer_auth(&intruder.token)
            .send()
            .await
            .expect("post");
        assert_eq!(attempt.status(), 404, "{suffix}");
    }

    let listed: Value = client
        .get(contracts_url(
            &app,
            intruder.organization_id,
            "?season=2026",
        ))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    assert!(listed["data"].as_array().unwrap().is_empty());

    let outside = client
        .get(contracts_url(
            &app,
            app.organization_id,
            &format!("/{contract_id}"),
        ))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("outside");
    assert_eq!(outside.status(), 403);

    let untouched: Value = client
        .get(contracts_url(
            &app,
            app.organization_id,
            &format!("/{contract_id}"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("untouched")
        .json()
        .await
        .expect("json");
    assert_eq!(untouched["data"], created["data"]);

    intruder.cleanup(&app.pool).await;
    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_ent_10_a_member_without_planning_manage_gets_403_on_every_contract_route() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let site = seed_site(&app, "GAUME", "Parc").await;
    let created = create_contract(&app, &site, vec![tonte()]).await;
    let contract_id = created["data"]["id"].as_str().unwrap().to_owned();
    let one = |suffix: &str| {
        contracts_url(
            &app,
            app.organization_id,
            &format!("/{contract_id}{suffix}"),
        )
    };
    let token = &app.no_permission_token;

    let list = client
        .get(contracts_url(&app, app.organization_id, "?season=2026"))
        .bearer_auth(token)
        .send()
        .await
        .expect("list");
    assert_eq!(list.status(), 403);

    let read = client
        .get(one(""))
        .bearer_auth(token)
        .send()
        .await
        .expect("read");
    assert_eq!(read.status(), 403);

    let create = client
        .post(contracts_url(&app, app.organization_id, ""))
        .bearer_auth(token)
        .json(&create_body(&site, vec![tonte()]))
        .send()
        .await
        .expect("create");
    assert_eq!(create.status(), 403);

    let replace = client
        .put(one(""))
        .bearer_auth(token)
        .json(&put_body(&created))
        .send()
        .await
        .expect("replace");
    assert_eq!(replace.status(), 403);

    for suffix in ["/archive", "/duplicate"] {
        let attempt = client
            .post(one(suffix))
            .bearer_auth(token)
            .send()
            .await
            .expect("post");
        assert_eq!(attempt.status(), 403, "{suffix}");
    }

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM maintenance_contracts WHERE org_id = $1")
            .bind(app.organization_id)
            .fetch_one(&app.pool)
            .await
            .expect("count");
    assert_eq!(count, 1, "no refused call changed anything");

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_ent_11_duplicating_copies_the_lines_into_the_next_season_without_interventions() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let site = seed_site(&app, "GAUME", "Parc").await;
    let created = create_contract(&app, &site, vec![tonte(), taille()]).await;
    let contract_id = created["data"]["id"].as_str().unwrap().to_owned();
    seed_interventions(
        &app,
        created["data"]["lines"][0]["id"].as_str().unwrap(),
        12,
    )
    .await;

    let duplicated = client
        .post(contracts_url(
            &app,
            app.organization_id,
            &format!("/{contract_id}/duplicate"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("duplicate");
    assert_eq!(duplicated.status(), 201);
    let copy: Value = duplicated.json().await.expect("json");
    let data = &copy["data"];

    assert_eq!(data["season"]["year"], 2027);
    assert_eq!(data["reference"], "C25029");
    assert_ne!(data["id"], created["data"]["id"]);
    assert_ne!(data["project_id"], created["data"]["project_id"]);
    let lines = data["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["label"], "Tonte avec ramassage");
    assert_eq!(lines[1]["cadence"], created["data"]["lines"][1]["cadence"]);
    assert!(
        lines
            .iter()
            .zip(created["data"]["lines"].as_array().unwrap())
            .all(|(copied, original)| copied["id"] != original["id"])
    );

    let copied_interventions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM maintenance_interventions i \
         JOIN maintenance_lines l ON l.id = i.line_id WHERE l.contract_id = $1",
    )
    .bind(data["id"].as_str().unwrap().parse::<Uuid>().unwrap())
    .fetch_one(&app.pool)
    .await
    .expect("count the copy's interventions");
    assert_eq!(copied_interventions, 0);

    let original_interventions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM maintenance_interventions i \
         JOIN maintenance_lines l ON l.id = i.line_id WHERE l.contract_id = $1",
    )
    .bind(contract_id.parse::<Uuid>().unwrap())
    .fetch_one(&app.pool)
    .await
    .expect("count the original's interventions");
    assert_eq!(original_interventions, 12, "the original keeps its history");

    let project_name: String = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1")
        .bind(
            data["project_id"]
                .as_str()
                .unwrap()
                .parse::<Uuid>()
                .unwrap(),
        )
        .fetch_one(&app.pool)
        .await
        .expect("the copy's project exists");
    assert_eq!(project_name, "Entretien 2027 - GAUME - Parc");

    cleanup(&app).await;
}
