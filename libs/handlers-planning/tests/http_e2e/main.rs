//! End-to-end tests for the manager's half of the correction loop:
//! `GET .../assignment-reports` and `PATCH .../assignment-reports/{id}/resolution`.
//!
//! A request enters over a real socket and crosses the whole stack: typed
//! routing, the rate-limit and auth middlewares, membership resolution, the
//! use case, a postgres transaction, and the durable event it commits.
//! Nothing in that chain is doubled except the identity provider, and even
//! that is a real HTTP server publishing a real JWKS — mirrors
//! `libs/handlers-field/tests/http_e2e/main.rs`.
//!
//! ```bash
//! docker compose up -d postgres redis
//! source .env
//! cargo test -p handlers-planning --test http_e2e -- --ignored
//! ```

mod harness;
mod issuer;

use serde_json::json;

/// The list defaults to pending, and resolving moves a report out of it —
/// without ever touching the task.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_list_defaults_to_pending_and_resolving_moves_a_report_out_of_it() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let anonymous = client
        .get(app.reports_url(""))
        .send()
        .await
        .expect("the api answers an unauthenticated call");
    assert_eq!(
        anonymous.status(),
        401,
        "the auth middleware must be in the chain"
    );

    let report_id = app.seed_pending_report(300).await;

    let pending: serde_json::Value = client
        .get(app.reports_url(""))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");
    let ids: Vec<&str> = pending["data"]
        .as_array()
        .expect("the list carries an array")
        .iter()
        .filter_map(|report| report["id"].as_str())
        .collect();
    assert!(
        ids.contains(&report_id.to_string().as_str()),
        "the default list must include the pending report: {pending}"
    );

    let resolved = client
        .patch(app.resolution_url(&report_id.to_string()))
        .bearer_auth(&app.token)
        .json(&json!({ "resolution": "APPLIED", "resolution_note": "Écart confirmé" }))
        .send()
        .await
        .expect("the api answers the resolution call");
    let status = resolved.status();
    let raw = resolved
        .text()
        .await
        .expect("the resolution answer has a body");
    assert!(status.is_success(), "resolving failed with {status}: {raw}");
    let resolved: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("the resolution answer is json: {e}: {raw}"));
    assert_eq!(
        resolved["data"]["resolution"],
        json!("APPLIED"),
        "{resolved}"
    );
    assert!(resolved["data"]["resolved_at"].is_string(), "{resolved}");

    // Gone from the default (pending) view, once resolved.
    let after: serde_json::Value = client
        .get(app.reports_url(""))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");
    let after_ids: Vec<&str> = after["data"]
        .as_array()
        .expect("the list carries an array")
        .iter()
        .filter_map(|report| report["id"].as_str())
        .collect();
    assert!(
        !after_ids.contains(&report_id.to_string().as_str()),
        "a resolved report must leave the default pending view: {after}"
    );

    // But still findable by explicitly asking for what it became.
    let applied: serde_json::Value = client
        .get(app.reports_url("?resolution=APPLIED"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the filtered list call")
        .json()
        .await
        .expect("the filtered list answer is json");
    let applied_ids: Vec<&str> = applied["data"]
        .as_array()
        .expect("the list carries an array")
        .iter()
        .filter_map(|report| report["id"].as_str())
        .collect();
    assert!(
        applied_ids.contains(&report_id.to_string().as_str()),
        "?resolution=APPLIED must surface the report it was just resolved into: {applied}"
    );

    app.cleanup().await;
}

/// Resolving twice must fail loudly rather than silently no-op — the domain
/// rule `AssignmentReportService::resolve_report` exists to enforce.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn resolving_an_already_resolved_report_is_refused() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let report_id = app.seed_pending_report(120).await;

    let first = client
        .patch(app.resolution_url(&report_id.to_string()))
        .bearer_auth(&app.token)
        .json(&json!({ "resolution": "DISMISSED", "resolution_note": null }))
        .send()
        .await
        .expect("the api answers the first resolution call");
    assert!(first.status().is_success(), "{}", first.status());

    let second = client
        .patch(app.resolution_url(&report_id.to_string()))
        .bearer_auth(&app.token)
        .json(&json!({ "resolution": "APPLIED", "resolution_note": null }))
        .send()
        .await
        .expect("the api answers the second resolution call");
    assert_eq!(
        second.status(),
        409,
        "a report already resolved must not resolve again"
    );

    app.cleanup().await;
}

/// `PENDING` is not a resolution a manager can decide *into*.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn resolving_into_pending_is_refused() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let report_id = app.seed_pending_report(90).await;

    let attempt = client
        .patch(app.resolution_url(&report_id.to_string()))
        .bearer_auth(&app.token)
        .json(&json!({ "resolution": "PENDING", "resolution_note": null }))
        .send()
        .await
        .expect("the api answers the resolution call");
    assert_eq!(attempt.status(), 409, "PENDING is not a valid target");

    app.cleanup().await;
}
