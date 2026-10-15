//! The field's half of the visit loop, end to end: declaring a maintenance
//! visit done on one's own task. @spec-ent-7 (the proposal) and @spec-ent-8
//! (somebody else's task is a 404).

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::harness;

/// A contract with one rolling line, and both fixture tasks made visits of
/// that line. Returns the line id.
async fn seed_maintenance_tasks(app: &harness::App) -> Uuid {
    let pool = &app.pool;
    let year = chrono::Datelike::iso_week(&Utc::now().date_naive()).year();

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
    let line_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO maintenance_lines (id, org_id, contract_id, label, cadence, position) \
         VALUES ($1, $2, $3, 'Tonte', $4, 0)",
    )
    .bind(line_id)
    .bind(app.organization_id)
    .bind(contract_id)
    .bind(json!({ "kind": "ROLLING", "every_days": 14 }))
    .execute(pool)
    .await
    .expect("seed the line");
    sqlx::query("UPDATE tasks SET maintenance_line_id = $2 WHERE id = ANY($1)")
        .bind(vec![app.task_id, app.other_task_id])
        .bind(line_id)
        .execute(pool)
        .await
        .expect("make the fixture tasks maintenance visits");

    line_id
}

async fn cleanup(app: &harness::App) {
    for statement in [
        "DELETE FROM maintenance_interventions WHERE org_id = $1",
        "UPDATE tasks SET maintenance_line_id = NULL WHERE org_id = $1",
        "DELETE FROM maintenance_lines WHERE org_id = $1",
        "DELETE FROM maintenance_contracts WHERE org_id = $1",
        "DELETE FROM projects WHERE org_id = $1",
        "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
    ] {
        sqlx::query(statement)
            .bind(app.organization_id)
            .execute(&app.pool)
            .await
            .unwrap_or_else(|e| panic!("maintenance cleanup failed on `{statement}`: {e}"));
    }
    app.cleanup().await;
}

fn intervention_url(app: &harness::App, task_id: Uuid) -> String {
    app.url(&format!("/tasks/{task_id}/intervention"))
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_worker_proposes_a_visit_on_their_own_task_and_only_there() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let line_id = seed_maintenance_tasks(&app).await;
    let today = Utc::now().date_naive();

    let anonymous = client
        .post(intervention_url(&app, app.task_id))
        .json(&json!({ "done_on": today }))
        .send()
        .await
        .expect("the api answers an unauthenticated call");
    assert_eq!(anonymous.status(), 401);

    let someone_elses = client
        .post(intervention_url(&app, app.other_task_id))
        .bearer_auth(&app.token)
        .json(&json!({ "done_on": today }))
        .send()
        .await
        .expect("the api answers the call on somebody else's task");
    assert_eq!(someone_elses.status(), 404);

    let unknown = client
        .post(intervention_url(&app, Uuid::now_v7()))
        .bearer_auth(&app.token)
        .json(&json!({ "done_on": today }))
        .send()
        .await
        .expect("the api answers the call on an unknown task");
    assert_eq!(unknown.status(), 404);

    let future = client
        .post(intervention_url(&app, app.task_id))
        .bearer_auth(&app.token)
        .json(&json!({ "done_on": today + chrono::Duration::days(5) }))
        .send()
        .await
        .expect("the api answers the call dated in the future");
    assert_eq!(future.status(), 409);

    let proposed = client
        .post(intervention_url(&app, app.task_id))
        .bearer_auth(&app.token)
        .json(&json!({ "done_on": today }))
        .send()
        .await
        .expect("the api answers the proposal");
    let status = proposed.status();
    let raw = proposed.text().await.expect("the proposal has a body");
    assert_eq!(status, 201, "{raw}");
    let proposed: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("json: {e}: {raw}"));
    assert_eq!(proposed["data"]["status"], json!("PROPOSED"), "{proposed}");
    assert_eq!(proposed["data"]["line_id"], json!(line_id), "{proposed}");
    assert_eq!(
        proposed["data"]["task_id"],
        json!(app.task_id),
        "{proposed}"
    );

    let rows: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT task_id, status FROM maintenance_interventions WHERE org_id = $1")
            .bind(app.organization_id)
            .fetch_all(&app.pool)
            .await
            .expect("read the interventions");
    assert_eq!(rows, vec![(app.task_id, "PROPOSED".to_owned())]);

    let events: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM automation.event WHERE org_id = $1 AND name LIKE 'maintenance_intervention.%'",
    )
    .bind(app.organization_id)
    .fetch_all(&app.pool)
    .await
    .expect("read the events");
    assert_eq!(events, ["maintenance_intervention.proposed"]);

    let tasks: serde_json::Value = client
        .get(app.url("/tasks"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the task list")
        .json()
        .await
        .expect("the task list is json");
    let mine = tasks["data"]
        .as_array()
        .and_then(|list| list.iter().find(|task| task["id"] == json!(app.task_id)))
        .expect("the caller's job is listed");
    assert_eq!(mine["maintenance_line_id"], json!(line_id), "{tasks}");

    cleanup(&app).await;
}
