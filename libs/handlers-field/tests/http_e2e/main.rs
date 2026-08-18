//! End-to-end tests for the field API.
//!
//! A request enters over a real socket and crosses the whole stack: typed
//! routing, the rate-limit and auth middlewares, the identity resolution from
//! account to member to employee, the use case, a postgres transaction, and
//! the events it commits. Nothing in that chain is doubled except the identity
//! provider, and even that is a real HTTP server publishing a real JWKS.
//!
//! ```bash
//! docker compose up -d postgres redis rustfs
//! source .env
//! cargo test -p handlers-field --test http_e2e -- --ignored
//! ```

mod harness;
mod issuer;

use serde_json::json;

/// A working day, start to finish, through the API.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_worker_clocks_on_photographs_the_job_and_ends_the_day() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let anonymous = client
        .get(app.url("/current"))
        .send()
        .await
        .expect("the api answers an unauthenticated call");
    assert_eq!(
        anonymous.status(),
        401,
        "the auth middleware must be in the chain"
    );

    // Only the caller's own job comes back, never the colleague's.
    let tasks: serde_json::Value = client
        .get(app.url("/tasks"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the task list")
        .json()
        .await
        .expect("the task list is json");
    let ids: Vec<&str> = tasks["data"]
        .as_array()
        .expect("the list carries an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    assert!(
        ids.contains(&app.task_id.to_string().as_str()),
        "the caller's own job is missing from {tasks}"
    );
    assert!(
        !ids.contains(&app.other_task_id.to_string().as_str()),
        "a colleague's job leaked into the caller's list: {tasks}"
    );

    // Nothing running yet, so the screen would offer "start".
    let current: serde_json::Value = client
        .get(app.url("/current"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers /current")
        .json()
        .await
        .expect("/current is json");
    assert_eq!(current["data"], json!(null), "{current}");

    let started = client
        .post(app.url("/time-entries"))
        .bearer_auth(&app.token)
        .json(&json!({ "task_id": app.task_id }))
        .send()
        .await
        .expect("the api answers the start call");
    let status = started.status();
    let raw = started.text().await.expect("the start answer has a body");
    assert!(
        status.is_success(),
        "clocking on failed with {status}: {raw}"
    );
    let started: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("the start answer is json: {e}: {raw}"));
    let entry_id = started["data"]["id"]
        .as_str()
        .expect("an entry id")
        .to_owned();
    assert_eq!(started["data"]["ended_at"], json!(null), "{started}");
    assert_eq!(started["data"]["worked_minutes"], json!(null), "{started}");

    // The rule the field app rests on.
    let second = client
        .post(app.url("/time-entries"))
        .bearer_auth(&app.token)
        .json(&json!({ "task_id": app.other_task_id }))
        .send()
        .await
        .expect("the api answers the second start");
    assert_eq!(
        second.status(),
        409,
        "a second job must be refused while one is running"
    );

    let photo = client
        .post(app.entry_url(&entry_id, "/photos"))
        .bearer_auth(&app.token)
        .json(&json!({ "phase": "BEFORE", "storage_key": "uploads/field/before.jpg" }))
        .send()
        .await
        .expect("the api answers the photo call");
    assert!(photo.status().is_success(), "{}", photo.status());

    // Ending the day closes what was left running, and the entry comes back
    // with the minutes the server computed.
    let ended = client
        .post(app.url("/day-end"))
        .bearer_auth(&app.token)
        .json(&json!({}))
        .send()
        .await
        .expect("the api answers the day-end call");
    let status = ended.status();
    let raw = ended.text().await.expect("the day-end answer has a body");
    assert!(
        status.is_success(),
        "ending the day failed with {status}: {raw}"
    );

    let after: serde_json::Value = client
        .get(app.url("/current"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers /current")
        .json()
        .await
        .expect("/current is json");
    assert_eq!(
        after["data"],
        json!(null),
        "the day is over, so nothing should still be running: {after}"
    );

    app.cleanup().await;
}
