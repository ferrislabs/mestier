//! WS6 of the Entretien chantier: what an MCP client can do with maintenance
//! contracts. A delegated token drafts a contract, reads it and reads the
//! recap; it never activates anything, and a draft is not in the recap until a
//! person activates it.

use chrono::{Datelike, NaiveDate, Utc};
use serde_json::{Value, json};

use crate::{
    harness, issuer,
    maintenance_contract::{Site, cleanup, seed_site_in},
};

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

fn iso_year_and_week() -> (i32, u32, String) {
    let iso = today().iso_week();
    (
        iso.year(),
        iso.week(),
        format!("{}-W{:02}", iso.year(), iso.week()),
    )
}

fn last_week_of(year: i32) -> u32 {
    if NaiveDate::from_isoywd_opt(year, 53, chrono::Weekday::Mon).is_some() {
        53
    } else {
        52
    }
}

fn contracts_url(app: &harness::App, suffix: &str) -> String {
    format!(
        "{}/api/v1/organizations/{}/maintenance/contracts{suffix}",
        app.base_url, app.organization_id
    )
}

fn recap_url(app: &harness::App, week: &str) -> String {
    format!(
        "{}/api/v1/organizations/{}/maintenance/recap?week={week}",
        app.base_url, app.organization_id
    )
}

/// A contract for the current ISO year whose single rolling line has never
/// been done, so an active one is overdue and sits in this week's recap.
fn body(site: &Site, label: &str, status: Option<&str>) -> Value {
    let (year, _, _) = iso_year_and_week();
    let mut body = json!({
        "customer_id": site.customer_id,
        "customer_context_id": site.context_id,
        "season": { "year": year, "first_week": 1, "last_week": last_week_of(year) },
        "lines": [{ "label": label, "cadence": { "kind": "ROLLING", "every_days": 14 } }]
    });
    if let Some(status) = status {
        body["status"] = json!(status);
    }
    body
}

async fn send(request: reqwest::RequestBuilder) -> (u16, Value) {
    let response = request.send().await.expect("the api answers");
    let status = response.status().as_u16();
    let raw = response.text().await.expect("the answer has a body");
    (
        status,
        serde_json::from_str(&raw).unwrap_or(Value::String(raw)),
    )
}

fn labels(recap: &Value) -> Vec<String> {
    recap["data"]["groups"]
        .as_array()
        .expect("the recap has groups")
        .iter()
        .flat_map(|group| group["lines"].as_array().expect("a group has lines"))
        .filter_map(|line| line["label"].as_str().map(str::to_owned))
        .collect()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_draft_made_with_a_delegated_token_stays_out_of_the_recap_until_a_person_activates_it() {
    let app = harness::start().await;
    sqlx::query("UPDATE organizations SET mcp_enabled = true WHERE id = $1")
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .expect("let MCP in");
    let client = reqwest::Client::new();
    let delegated = issuer::mint_delegated(&app.manager_sub);
    let (_, _, week) = iso_year_and_week();
    let imported = seed_site_in(&app, "GAUME", "Jardin", "Vichy").await;
    let running = seed_site_in(&app, "MARTIN", "Parc", "Vichy").await;

    let (status, active) = send(
        client
            .post(contracts_url(&app, ""))
            .bearer_auth(&app.token)
            .json(&body(&running, "Tonte du parc", None)),
    )
    .await;
    assert_eq!(status, 201, "{active}");

    let (status, draft) = send(
        client
            .post(contracts_url(&app, ""))
            .bearer_auth(&delegated)
            .json(&body(&imported, "Tonte du jardin", Some("DRAFT"))),
    )
    .await;
    assert_eq!(status, 201, "{draft}");
    assert_eq!(draft["data"]["status"], "DRAFT", "{draft}");
    assert_eq!(draft["data"]["lines"].as_array().map(Vec::len), Some(1));
    let draft_id = draft["data"]["id"].as_str().expect("the draft has an id");

    let (status, recap) = send(client.get(recap_url(&app, &week)).bearer_auth(&delegated)).await;
    assert_eq!(status, 200, "{recap}");
    assert_eq!(
        labels(&recap),
        ["Tonte du parc"],
        "the active contract is due, the draft is not: {recap}"
    );

    let (status, read) = send(
        client
            .get(contracts_url(&app, &format!("/{draft_id}")))
            .bearer_auth(&delegated),
    )
    .await;
    assert_eq!(status, 200, "{read}");
    assert_eq!(read["data"], draft["data"]);

    let (status, refused) = send(
        client
            .put(contracts_url(&app, &format!("/{draft_id}")))
            .bearer_auth(&delegated)
            .json(&json!({
                "season": draft["data"]["season"],
                "status": "ACTIVE",
                "lines": draft["data"]["lines"],
            })),
    )
    .await;
    assert_eq!(status, 403, "{refused}");
    assert_eq!(refused["code"], "E_DELEGATED_ROUTE_FORBIDDEN", "{refused}");

    let (status, refused) = send(
        client
            .post(contracts_url(&app, ""))
            .bearer_auth(&delegated)
            .json(&body(&imported, "Born active", Some("ACTIVE"))),
    )
    .await;
    assert_eq!(status, 403, "{refused}");
    let (status, refused) = send(
        client
            .post(contracts_url(&app, ""))
            .bearer_auth(&delegated)
            .json(&body(&imported, "Status left out", None)),
    )
    .await;
    assert_eq!(status, 403, "a missing status means ACTIVE: {refused}");

    let (_, still) = send(client.get(recap_url(&app, &week)).bearer_auth(&app.token)).await;
    assert_eq!(labels(&still), ["Tonte du parc"], "{still}");

    let mut activation = body(&imported, "Tonte du jardin", Some("ACTIVE"));
    activation["season"] = draft["data"]["season"].clone();
    activation["lines"] = draft["data"]["lines"].clone();
    let (status, activated) = send(
        client
            .put(contracts_url(&app, &format!("/{draft_id}")))
            .bearer_auth(&app.token)
            .json(&activation),
    )
    .await;
    assert_eq!(status, 200, "{activated}");
    assert_eq!(activated["data"]["status"], "ACTIVE", "{activated}");

    let (_, after) = send(client.get(recap_url(&app, &week)).bearer_auth(&app.token)).await;
    let mut listed = labels(&after);
    listed.sort();
    assert_eq!(listed, ["Tonte du jardin", "Tonte du parc"], "{after}");

    cleanup(&app).await;
}
