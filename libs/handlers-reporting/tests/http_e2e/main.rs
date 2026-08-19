//! End-to-end tests for the reporting API.
//!
//! Real socket, real database, real auth. The fixture clocks three hours on a
//! chantier quoted at 4 200 €, at an employee rate of 35 €/h, so the numbers
//! the report returns are ones this file can state rather than merely accept.
//!
//! ```bash
//! docker compose up -d postgres redis rustfs
//! source .env
//! cargo test -p handlers-reporting --test http_e2e -- --ignored
//! ```

mod harness;
mod issuer;

use chrono::Utc;

fn period() -> String {
    let today = Utc::now().date_naive();
    let from = today - chrono::Duration::days(2);
    format!("from={from}&to={today}")
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_worked_chantier_is_costed_and_compared_to_its_quote() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let anonymous = client
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .send()
        .await
        .expect("the api answers an unauthenticated call");
    assert_eq!(
        anonymous.status(),
        401,
        "the auth middleware must be in the chain"
    );

    let raw = client
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the profitability call")
        .text()
        .await
        .expect("the answer has a body");
    let body: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("the profitability answer is json: {e}: {raw}"));

    let job = body["data"]["jobs"]
        .as_array()
        .expect("the report carries jobs")
        .iter()
        .find(|job| job["task_id"] == serde_json::json!(app.task_id))
        .unwrap_or_else(|| panic!("the worked chantier is missing from {body}"));

    // Three hours at 35 euros an hour, and no equipment on the fixture.
    assert_eq!(job["worked_minutes"], serde_json::json!(180), "{job}");
    assert_eq!(job["labour_cost_cents"], serde_json::json!(10_500), "{job}");
    assert_eq!(job["equipment_cost_cents"], serde_json::json!(0), "{job}");
    assert_eq!(job["quoted_cents"], serde_json::json!(420_000), "{job}");
    assert_eq!(
        job["margin_cents"],
        serde_json::json!(420_000 - 10_500),
        "the margin is the quote less what it cost: {job}"
    );

    // The chantier nobody clocked on has no cost to report, so it is absent
    // rather than listed at zero.
    let other = body["data"]["jobs"]
        .as_array()
        .expect("the report carries jobs")
        .iter()
        .any(|job| job["task_id"] == serde_json::json!(app.other_task_id));
    assert!(
        !other,
        "a chantier with no clocked time should not be reported: {body}"
    );

    // Ranked server-side, so every screen agrees on what "most profitable" is.
    assert!(
        !body["data"]["most_profitable"]
            .as_array()
            .expect("a ranking")
            .is_empty(),
        "{body}"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn worked_hours_reports_the_same_time_the_costing_used() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let body: serde_json::Value = client
        .get(format!("{}?{}", app.url("/worked-hours"), period()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the worked-hours call")
        .json()
        .await
        .expect("the answer is json");

    assert_eq!(
        body["data"]["total_worked_minutes"],
        serde_json::json!(180),
        "payroll and costing must not disagree about the same week: {body}"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_period_that_ends_before_it_starts_is_refused() {
    let app = harness::start().await;

    let answer = reqwest::Client::new()
        .get(format!(
            "{}?from=2026-08-20&to=2026-08-01",
            app.url("/profitability")
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the call");

    assert_eq!(answer.status(), 409);

    app.cleanup().await;
}
