//! End-to-end tests for an employee's cost history.
//!
//! Real socket, real database, real auth. The fixture seeds one organization
//! with an hourly employee already on an open cost basis version, plus a
//! second organization used only as the target of the cross-tenant refusal
//! test.

mod harness;
mod issuer;

use chrono::{Duration, NaiveDate, Utc};

fn future_date() -> NaiveDate {
    Utc::now().date_naive() + Duration::days(30)
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_cost_basis_is_dated_listed_and_then_corrected() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let effective_from = future_date();

    let created: serde_json::Value = client
        .post(app.url(&format!("/employees/{}/cost-bases", app.employee_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "effective_from": effective_from,
            "is_salaried": false,
            "hourly_rate_cents": 4_200,
            "weekly_contract_minutes": 2100,
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(
        created["data"]["hourly_rate_cents"],
        serde_json::json!(4_200),
        "{created}"
    );
    assert_eq!(
        created["data"]["effective_from"],
        serde_json::json!(effective_from),
        "{created}"
    );
    let new_id = created["data"]["id"]
        .as_str()
        .expect("the created version has an id")
        .to_owned();

    let history: serde_json::Value = client
        .get(app.url(&format!("/employees/{}/cost-bases", app.employee_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the answer is json");
    let rows = history["data"].as_array().expect("the history is an array");
    assert_eq!(
        rows.len(),
        2,
        "the original version plus the dated one: {history}"
    );
    assert_eq!(
        rows[0]["id"],
        serde_json::json!(app.cost_basis_id),
        "the fixture's own version must still be first, oldest first: {history}"
    );
    assert_eq!(
        rows[0]["effective_to"],
        serde_json::json!(effective_from),
        "dating a change must close the version it follows: {history}"
    );

    let corrected: serde_json::Value = client
        .patch(app.url(&format!("/cost-bases/{new_id}")))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "effective_from": effective_from,
            "effective_to": null,
            "is_salaried": false,
            "hourly_rate_cents": 4_500,
            "weekly_contract_minutes": 2100,
        }))
        .send()
        .await
        .expect("the api answers the correction call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(
        corrected["data"]["hourly_rate_cents"],
        serde_json::json!(4_500),
        "a correction rewrites the version in place: {corrected}"
    );

    app.cleanup().await;
}

/// The overlap refusal: correcting a version to start before the one it
/// followed must surface as a 409, not the 500 `map_sqlx_error` used to give
/// the exclusion constraint before #302.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn correcting_a_version_to_overlap_another_is_refused_as_a_conflict() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(app.url(&format!("/employees/{}/cost-bases", app.employee_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "effective_from": future_date(),
            "is_salaried": false,
            "hourly_rate_cents": 4_200,
            "weekly_contract_minutes": 2100,
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the answer is json");
    let new_id = created["data"]["id"]
        .as_str()
        .expect("the created version has an id")
        .to_owned();

    // Corrects the new version to start back in 2020 — squarely inside the
    // range the original, still-open version covers.
    let response = client
        .patch(app.url(&format!("/cost-bases/{new_id}")))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "effective_from": "2020-06-01",
            "effective_to": null,
            "is_salaried": false,
            "hourly_rate_cents": 4_200,
            "weekly_contract_minutes": 2100,
        }))
        .send()
        .await
        .expect("the api answers the correction call");

    assert_eq!(
        response.status(),
        409,
        "an overlapping correction must be refused"
    );

    app.cleanup().await;
}

/// The cross-tenant refusal: a bare `employee_id`/`cost_basis_id` from
/// another organization must never be reachable through this token, whatever
/// the exact status code — the loaded row's own organization is what
/// authorization runs against, never the caller's.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_cost_basis_from_another_organization_is_not_reachable() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let list_response = client
        .get(app.url(&format!("/employees/{}/cost-bases", app.other_employee_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call");
    assert_ne!(
        list_response.status(),
        200,
        "another organization's history must not be readable"
    );

    let correct_response = client
        .patch(app.url(&format!("/cost-bases/{}", app.other_cost_basis_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "effective_from": "2020-01-01",
            "effective_to": null,
            "is_salaried": false,
            "hourly_rate_cents": 9_999,
            "weekly_contract_minutes": 2100,
        }))
        .send()
        .await
        .expect("the api answers the correction call");
    assert_ne!(
        correct_response.status(),
        200,
        "another organization's version must not be correctable"
    );

    app.cleanup().await;
}
