//! The manager's half of the visit loop and the two read views, end to end:
//! entering, validating and rejecting visits, the season grid, the weekly
//! recap and planning from it. @spec-ent-5, @spec-ent-6, @spec-ent-7 (the
//! validation), @spec-ent-9 and @spec-ent-10 over HTTP; the rules themselves
//! are covered in `mestier-core`.

use chrono::{Datelike, NaiveDate, Utc};
use mestier_core::Permissions;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{harness, issuer};

struct Seeded {
    year: i32,
    project_id: Uuid,
    customer_id: Uuid,
    context_id: Uuid,
    /// "Tonte" then "Taille", both rolling every 14 days.
    line_ids: [Uuid; 2],
}

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

async fn seed_contract(app: &harness::App) -> Seeded {
    let year = today().iso_week().year();
    let pool = &app.pool;

    let customer_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, 'GAUME')")
        .bind(customer_id)
        .bind(app.organization_id)
        .execute(pool)
        .await
        .expect("seed the customer");
    let context_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO customer_contexts (id, customer_id, label, city) \
         VALUES ($1, $2, 'Jardin GAUME', 'Vichy')",
    )
    .bind(context_id)
    .bind(customer_id)
    .execute(pool)
    .await
    .expect("seed the site");
    let project_id = Uuid::now_v7();
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, 'Entretien GAUME')")
        .bind(project_id)
        .bind(app.organization_id)
        .execute(pool)
        .await
        .expect("seed the project");
    let contract_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO maintenance_contracts \
         (id, org_id, customer_id, customer_context_id, project_id, season_year, first_week, last_week) \
         VALUES ($1, $2, $3, $4, $5, $6, 1, 52)",
    )
    .bind(contract_id)
    .bind(app.organization_id)
    .bind(customer_id)
    .bind(context_id)
    .bind(project_id)
    .bind(year as i16)
    .execute(pool)
    .await
    .expect("seed the contract");

    let mut line_ids = [Uuid::now_v7(), Uuid::now_v7()];
    for (position, (line_id, label)) in line_ids.iter_mut().zip(["Tonte", "Taille"]).enumerate() {
        sqlx::query(
            "INSERT INTO maintenance_lines (id, org_id, contract_id, label, cadence, position) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(*line_id)
        .bind(app.organization_id)
        .bind(contract_id)
        .bind(label)
        .bind(json!({ "kind": "ROLLING", "every_days": 14 }))
        .bind(position as i32)
        .execute(pool)
        .await
        .expect("seed the line");
    }

    Seeded {
        year,
        project_id,
        customer_id,
        context_id,
        line_ids,
    }
}

/// A visit straight into the table, proposed by the fixture's assignee.
async fn seed_proposed_visit(app: &harness::App, line_id: Uuid, done_on: NaiveDate) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO maintenance_interventions (id, org_id, line_id, done_on, status, proposed_by) \
         VALUES ($1, $2, $3, $4, 'PROPOSED', $5)",
    )
    .bind(id)
    .bind(app.organization_id)
    .bind(line_id)
    .bind(done_on)
    .bind(app.assignee_member_id)
    .execute(&app.pool)
    .await
    .expect("seed the proposed visit");
    id
}

async fn cleanup(app: &harness::App) {
    for statement in [
        "DELETE FROM maintenance_interventions WHERE org_id = $1",
        "DELETE FROM tasks WHERE org_id = $1",
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

fn url(organization_id: Uuid, base: &str, suffix: &str) -> String {
    format!("{base}/api/v1/organizations/{organization_id}/maintenance{suffix}")
}

fn app_url(app: &harness::App, suffix: &str) -> String {
    url(app.organization_id, &app.base_url, suffix)
}

async fn body(response: reqwest::Response) -> (u16, Value) {
    let status = response.status().as_u16();
    let raw = response.text().await.expect("the answer has a body");
    let json = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
    (status, json)
}

/// The ISO week of `date` as the API writes it.
fn iso_week(date: NaiveDate) -> String {
    let week = date.iso_week();
    format!("{}-W{:02}", week.year(), week.week())
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_manager_records_validates_and_rejects_visits() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let seeded = seed_contract(&app).await;
    let [tonte, taille] = seeded.line_ids;

    let recorded = client
        .post(app_url(&app, &format!("/lines/{tonte}/interventions")))
        .bearer_auth(&app.token)
        .json(&json!({ "done_on": today() }))
        .send()
        .await
        .expect("the api answers the record call");
    let (status, recorded) = body(recorded).await;
    assert_eq!(status, 201, "{recorded}");
    assert_eq!(recorded["data"]["status"], "VALIDATED", "{recorded}");
    assert_eq!(recorded["data"]["task_id"], Value::Null, "{recorded}");

    let in_the_future = client
        .post(app_url(&app, &format!("/lines/{tonte}/interventions")))
        .bearer_auth(&app.token)
        .json(&json!({ "done_on": today() + chrono::Duration::days(3) }))
        .send()
        .await
        .expect("the api answers the future record call");
    assert_eq!(in_the_future.status(), 409);

    let to_validate = seed_proposed_visit(&app, taille, today()).await;
    let to_reject = seed_proposed_visit(&app, taille, today()).await;
    let to_reject_without_body = seed_proposed_visit(&app, taille, today()).await;

    let (status, queue) = body(
        client
            .get(app_url(&app, "/interventions?status=proposed"))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the list call"),
    )
    .await;
    assert_eq!(status, 200, "{queue}");
    assert_eq!(queue["data"].as_array().map(Vec::len), Some(3), "{queue}");
    let (_, default_queue) = body(
        client
            .get(app_url(&app, "/interventions"))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the default list call"),
    )
    .await;
    assert_eq!(default_queue["data"], queue["data"]);

    let validated = client
        .post(app_url(
            &app,
            &format!("/interventions/{to_validate}/validate"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the validate call");
    let (status, validated) = body(validated).await;
    assert_eq!(status, 200, "{validated}");
    assert_eq!(validated["data"]["status"], "VALIDATED", "{validated}");
    assert!(validated["data"]["resolved_by"].is_string(), "{validated}");

    let again = client
        .post(app_url(
            &app,
            &format!("/interventions/{to_validate}/validate"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the second validate call");
    assert_eq!(again.status(), 409);

    let rejected = client
        .post(app_url(&app, &format!("/interventions/{to_reject}/reject")))
        .bearer_auth(&app.token)
        .json(&json!({ "note": "Pas passé ce jour-là" }))
        .send()
        .await
        .expect("the api answers the reject call");
    let (status, rejected) = body(rejected).await;
    assert_eq!(status, 200, "{rejected}");
    assert_eq!(rejected["data"]["status"], "REJECTED", "{rejected}");
    assert_eq!(
        rejected["data"]["note"], "Pas passé ce jour-là",
        "{rejected}"
    );

    let rejected_bare = client
        .post(app_url(
            &app,
            &format!("/interventions/{to_reject_without_body}/reject"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the reject call without a body");
    assert_eq!(rejected_bare.status(), 200);

    let blank_note = client
        .post(app_url(
            &app,
            &format!("/interventions/{to_validate}/reject"),
        ))
        .bearer_auth(&app.token)
        .json(&json!({ "note": "  " }))
        .send()
        .await
        .expect("the api answers the blank note call");
    assert_eq!(blank_note.status(), 409);

    let (_, left) = body(
        client
            .get(app_url(&app, "/interventions?status=proposed"))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the list call"),
    )
    .await;
    assert_eq!(left["data"].as_array().map(Vec::len), Some(0), "{left}");

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_grid_the_recap_and_planning_from_it() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let seeded = seed_contract(&app).await;
    let [tonte, taille] = seeded.line_ids;

    let (status, grid) = body(
        client
            .get(app_url(&app, &format!("/grid?season={}", seeded.year)))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the grid call"),
    )
    .await;
    assert_eq!(status, 200, "{grid}");
    let contract = &grid["data"]["contracts"][0];
    assert_eq!(contract["site"]["customer_name"], "GAUME", "{grid}");
    assert_eq!(contract["site"]["city"], "Vichy", "{grid}");
    assert_eq!(contract["lines"][0]["label"], "Tonte", "{grid}");
    assert_eq!(contract["lines"][0]["due"]["status"], "OVERDUE", "{grid}");

    let week = iso_week(today());
    let (status, recap) = body(
        client
            .get(app_url(&app, &format!("/recap?week={week}")))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the recap call"),
    )
    .await;
    assert_eq!(status, 200, "{recap}");
    let listed: Vec<&str> = recap["data"]["groups"][0]["lines"]
        .as_array()
        .expect("the recap has a first town")
        .iter()
        .filter_map(|line| line["label"].as_str())
        .collect();
    assert_eq!(listed, ["Tonte", "Taille"], "{recap}");
    assert_eq!(recap["data"]["groups"][0]["town"], "Vichy", "{recap}");

    for bad in ["2026-43", "week", "2026-W54"] {
        let refused = client
            .get(app_url(&app, &format!("/recap?week={bad}")))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the malformed recap call");
        assert_eq!(refused.status(), 400, "{bad}");
    }
    let no_such_week = client
        .get(app_url(&app, "/recap?week=2027-W53"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the recap call for a week the year lacks");
    assert_eq!(no_such_week.status(), 409);

    let planned = client
        .post(app_url(&app, "/recap/plan"))
        .bearer_auth(&app.token)
        .json(&json!({ "line_ids": [tonte], "week": week }))
        .send()
        .await
        .expect("the api answers the plan call");
    let (status, planned) = body(planned).await;
    assert_eq!(status, 201, "{planned}");
    let task = &planned["data"][0];
    assert_eq!(task["title"], "Tonte", "{planned}");
    assert_eq!(task["status"], "BACKLOG", "{planned}");
    assert_eq!(task["maintenance_line_id"], json!(tonte), "{planned}");
    assert_eq!(task["project_id"], json!(seeded.project_id), "{planned}");
    assert_eq!(task["customer_id"], json!(seeded.customer_id), "{planned}");
    assert_eq!(
        task["customer_context_id"],
        json!(seeded.context_id),
        "{planned}"
    );
    assert_eq!(task["starts_at"], Value::Null, "{planned}");

    let again = client
        .post(app_url(&app, "/recap/plan"))
        .bearer_auth(&app.token)
        .json(&json!({ "line_ids": [tonte], "week": week }))
        .send()
        .await
        .expect("the api answers the second plan call");
    assert_eq!(again.status(), 409);

    let (_, after) = body(
        client
            .get(app_url(&app, &format!("/recap?week={week}")))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the recap call"),
    )
    .await;
    let still: Vec<&str> = after["data"]["groups"][0]["lines"]
        .as_array()
        .expect("the recap still has a town")
        .iter()
        .filter_map(|line| line["label"].as_str())
        .collect();
    assert_eq!(
        still,
        [taille.to_string().as_str()].map(|_| "Taille"),
        "{after}"
    );

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_ent_10_a_member_without_planning_manage_gets_403() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let seeded = seed_contract(&app).await;
    let [tonte, _] = seeded.line_ids;
    let visit = seed_proposed_visit(&app, tonte, today()).await;
    let week = iso_week(today());
    let token = &app.no_permission_token;

    let attempts = [
        client.get(app_url(&app, &format!("/grid?season={}", seeded.year))),
        client.get(app_url(&app, &format!("/recap?week={week}"))),
        client.get(app_url(&app, "/interventions")),
        client
            .post(app_url(&app, &format!("/lines/{tonte}/interventions")))
            .json(&json!({ "done_on": today() })),
        client.post(app_url(&app, &format!("/interventions/{visit}/validate"))),
        client.post(app_url(&app, &format!("/interventions/{visit}/reject"))),
        client
            .post(app_url(&app, "/recap/plan"))
            .json(&json!({ "line_ids": [tonte], "week": week })),
    ];
    for attempt in attempts {
        let request = attempt.bearer_auth(token).build().expect("a request");
        let (method, path) = (request.method().clone(), request.url().path().to_owned());
        let response = client.execute(request).await.expect("the api answers");
        assert_eq!(response.status(), 403, "{method} {path}");
    }

    let still_proposed: String =
        sqlx::query_scalar("SELECT status FROM maintenance_interventions WHERE id = $1")
            .bind(visit)
            .fetch_one(&app.pool)
            .await
            .expect("read the visit");
    assert_eq!(still_proposed, "PROPOSED");

    cleanup(&app).await;
}

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

/// A manager of another organization, holding `planning.manage` there, so a
/// 404 below can only come from the data belonging elsewhere.
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
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, 'Intruder')",
    )
    .bind(member_id)
    .bind(organization_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("seed the intruder's membership");
    let role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, 'manager', $3)",
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

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_ent_9_another_organization_gets_404_and_an_empty_grid() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let seeded = seed_contract(&app).await;
    let [tonte, _] = seeded.line_ids;
    let visit = seed_proposed_visit(&app, tonte, today()).await;
    let intruder = seed_intruder(&app.pool).await;
    let theirs = |suffix: &str| url(intruder.organization_id, &app.base_url, suffix);

    let attempts = [
        client
            .post(theirs(&format!("/lines/{tonte}/interventions")))
            .json(&json!({ "done_on": today() })),
        client.post(theirs(&format!("/interventions/{visit}/validate"))),
        client.post(theirs(&format!("/interventions/{visit}/reject"))),
        client
            .post(theirs("/recap/plan"))
            .json(&json!({ "line_ids": [tonte], "week": iso_week(today()) })),
    ];
    for attempt in attempts {
        let request = attempt
            .bearer_auth(&intruder.token)
            .build()
            .expect("a request");
        let (method, path) = (request.method().clone(), request.url().path().to_owned());
        let response = client.execute(request).await.expect("the api answers");
        assert_eq!(response.status(), 404, "{method} {path}");
    }

    let (status, grid) = body(
        client
            .get(theirs(&format!("/grid?season={}", seeded.year)))
            .bearer_auth(&intruder.token)
            .send()
            .await
            .expect("the api answers the intruder's grid call"),
    )
    .await;
    assert_eq!(status, 200, "{grid}");
    assert_eq!(grid["data"]["contracts"].as_array().map(Vec::len), Some(0));

    let untouched: String =
        sqlx::query_scalar("SELECT status FROM maintenance_interventions WHERE id = $1")
            .bind(visit)
            .fetch_one(&app.pool)
            .await
            .expect("read the visit");
    assert_eq!(untouched, "PROPOSED");

    intruder.cleanup(&app.pool).await;
    cleanup(&app).await;
}
