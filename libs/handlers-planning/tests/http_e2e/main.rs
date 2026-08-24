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

/// The acceptance criterion #294 exists for: a series is created and
/// materializes real tasks; editing one occurrence detaches it and the
/// response says so; deleting the series removes its future occurrences but
/// leaves the detached one — no longer part of any series — standing.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn editing_an_occurrence_detaches_it_and_deleting_the_series_leaves_it_standing() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let starts_on = chrono::Utc::now().date_naive();
    let created = client
        .post(app.recurrences_url(""))
        .bearer_auth(&app.token)
        .json(&json!({
            "frequency": "DAILY",
            "starts_on": starts_on.to_string(),
            "timezone": "Europe/Paris",
            "start_time": "09:00:00",
            "duration_minutes": 60,
            "all_day": false,
            "title": "Réunion hebdo",
            "blocks_availability": true,
            "assignee_member_ids": [app.assignee_member_id.to_string()],
        }))
        .send()
        .await
        .expect("the api answers the creation call");
    let status = created.status();
    let created: serde_json::Value = created
        .json()
        .await
        .unwrap_or_else(|e| panic!("the creation answer is json: {e}"));
    assert!(status.is_success(), "creation failed: {created}");
    let recurrence_id = created["data"]["id"]
        .as_str()
        .expect("the created recurrence carries an id")
        .to_owned();

    // The recurrence now shows up in the organization's list.
    let listed: serde_json::Value = client
        .get(app.recurrences_url(""))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");
    let listed_ids: Vec<&str> = listed["data"]
        .as_array()
        .expect("the list carries an array")
        .iter()
        .filter_map(|r| r["id"].as_str())
        .collect();
    assert!(listed_ids.contains(&recurrence_id.as_str()), "{listed}");

    // Real `tasks` rows exist — materialization actually happened.
    let occurrences = app.occurrence_task_ids(&recurrence_id).await;
    assert!(
        occurrences.len() > 1,
        "a daily recurrence materializes more than one occurrence: {occurrences:?}"
    );
    let first_occurrence = occurrences[0];
    let later_occurrence = *occurrences.last().unwrap();

    // Editing the first occurrence detaches it, and the response says so.
    let patched = client
        .patch(app.task_url(&first_occurrence.to_string()))
        .bearer_auth(&app.token)
        .json(&json!({ "title": "Réunion déplacée" }))
        .send()
        .await
        .expect("the api answers the patch call");
    let patch_status = patched.status();
    let patched: serde_json::Value = patched
        .json()
        .await
        .unwrap_or_else(|e| panic!("the patch answer is json: {e}"));
    assert!(patch_status.is_success(), "patch failed: {patched}");
    assert_eq!(
        patched["data"]["detached"],
        json!(true),
        "editing an occurrence must report that it detached: {patched}"
    );
    assert_eq!(
        patched["data"]["task"]["recurrence_id"],
        serde_json::Value::Null,
        "a detached task no longer names its series: {patched}"
    );

    // Deleting the series removes its future occurrences...
    let deleted = client
        .delete(app.recurrence_url(&recurrence_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete call");
    assert!(
        deleted.status().is_success(),
        "deleting the series failed: {}",
        deleted.status()
    );

    let later = client
        .get(app.task_url(&later_occurrence.to_string()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call");
    assert_eq!(
        later.status(),
        404,
        "a future occurrence must be gone once the series is deleted"
    );

    // ...but the detached occurrence — no longer part of any series — is
    // left standing.
    let standing = client
        .get(app.task_url(&first_occurrence.to_string()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call");
    let standing_status = standing.status();
    let standing: serde_json::Value = standing
        .json()
        .await
        .unwrap_or_else(|e| panic!("the get answer is json: {e}"));
    assert!(
        standing_status.is_success(),
        "the detached occurrence must still exist: {standing}"
    );
    assert_eq!(standing["data"]["title"], json!("Réunion déplacée"));

    app.cleanup().await;
}
