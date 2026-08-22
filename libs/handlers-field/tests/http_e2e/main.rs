//! End-to-end tests for the field API.
//!
//! A request enters over a real socket and crosses the whole stack: typed
//! routing, the rate-limit and auth middlewares, the identity resolution from
//! account to member to employee, the use case, a postgres transaction, and
//! the events it commits. Nothing in that chain is doubled except the identity
//! provider, and even that is a real HTTP server publishing a real JWKS.
//!
//! ```bash
//! docker compose up -d postgres redis
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
    assert_eq!(
        current["data"],
        json!({ "running": null, "day_ended_at": null }),
        "{current}"
    );

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
        after["data"]["running"],
        json!(null),
        "the day is over, so nothing should still be running: {after}"
    );
    assert!(
        after["data"]["day_ended_at"].is_string(),
        "the day-end just declared must be reflected on the next read: {after}"
    );

    app.cleanup().await;
}

/// The rush-hour case: work happened, nothing was ever clocked live, and
/// there is nothing open to recover — so the employee declares both ends at
/// once instead.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_stretch_never_clocked_live_is_declared_after_the_fact() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let now = chrono::Utc::now();
    let started_at = (now - chrono::Duration::hours(3)).to_rfc3339();
    let ended_at = (now - chrono::Duration::hours(2)).to_rfc3339();

    let raw = client
        .post(app.url("/time-entries/declare"))
        .bearer_auth(&app.token)
        .json(&json!({
            "task_id": app.task_id,
            "started_at": started_at,
            "ended_at": ended_at,
        }))
        .send()
        .await
        .expect("the api answers the declare call")
        .text()
        .await
        .expect("the declare answer has a body");
    let body: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("the declare answer is json: {e}: {raw}"));

    assert_eq!(
        body["data"]["worked_minutes"],
        json!(60),
        "one hour, as declared: {body}"
    );
    let returned_ended_at: chrono::DateTime<chrono::Utc> = body["data"]["ended_at"]
        .as_str()
        .expect("ended_at is a string")
        .parse()
        .expect("ended_at is a valid instant");
    assert_eq!(
        returned_ended_at,
        ended_at.parse::<chrono::DateTime<chrono::Utc>>().unwrap(),
        "closed the moment it was created: {body}"
    );

    // A declared end before its own start is refused, same rule as `stop`.
    let backwards = client
        .post(app.url("/time-entries/declare"))
        .bearer_auth(&app.token)
        .json(&json!({
            "task_id": app.task_id,
            "started_at": ended_at,
            "ended_at": started_at,
        }))
        .send()
        .await
        .expect("the api answers the declare call");
    assert_eq!(backwards.status(), 409, "an end before its own start");

    // A declared end in the future is refused: nobody can recollect work
    // that has not happened yet.
    let in_the_future = client
        .post(app.url("/time-entries/declare"))
        .bearer_auth(&app.token)
        .json(&json!({
            "task_id": app.task_id,
            "started_at": started_at,
            "ended_at": (now + chrono::Duration::hours(1)).to_rfc3339(),
        }))
        .send()
        .await
        .expect("the api answers the declare call");
    assert_eq!(in_the_future.status(), 409, "an end still in the future");

    // Clocking on live, then declaring a stretch reaching into that open job,
    // would double-count the same time — refused for the same reason `start`
    // refuses a second live job.
    let live = client
        .post(app.url("/time-entries"))
        .bearer_auth(&app.token)
        .json(&json!({ "task_id": app.task_id }))
        .send()
        .await
        .expect("the api answers the start call");
    assert!(live.status().is_success(), "{}", live.status());
    // Captured after the live entry started, so a declared end at this
    // instant is unambiguously later than `running.started_at` — otherwise
    // network latency between this test and the server could land the
    // declared end a hair before the live entry actually opened.
    let after_live_started = chrono::Utc::now();

    let overlapping = client
        .post(app.url("/time-entries/declare"))
        .bearer_auth(&app.token)
        .json(&json!({
            "task_id": app.task_id,
            "started_at": (after_live_started - chrono::Duration::hours(1)).to_rfc3339(),
            "ended_at": after_live_started.to_rfc3339(),
        }))
        .send()
        .await
        .expect("the api answers the declare call");
    assert_eq!(
        overlapping.status(),
        409,
        "a declared stretch reaching into the still-running job"
    );

    app.cleanup().await;
}

/// The forgotten clock-off, end to end.
///
/// A stretch begun thirty hours ago and never closed. Clocking off now would
/// record thirty hours nobody worked, so the API refuses and the employee is
/// asked for the real time instead.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_forgotten_stretch_is_recovered_at_the_declared_time_not_at_now() {
    let app = harness::start().await;
    let entry_id = harness::seed_forgotten_entry(&app.pool, &app).await;
    let client = reqwest::Client::new();

    // Clocking off now would turn a day's work into thirty hours.
    let stopped = client
        .post(app.entry_url(&entry_id.to_string(), "/stop"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the stop call");
    assert_eq!(
        stopped.status(),
        409,
        "a stretch from an earlier day must not be closed at today's time"
    );

    // Starting a new job is blocked too, which is what would strand the worker.
    let blocked = client
        .post(app.url("/time-entries"))
        .bearer_auth(&app.token)
        .json(&json!({ "task_id": app.task_id }))
        .send()
        .await
        .expect("the api answers the start call");
    assert_eq!(blocked.status(), 409, "one open stretch at a time");

    // The employee says when they actually finished.
    let declared = (chrono::Utc::now() - chrono::Duration::hours(23)).to_rfc3339();
    let raw = client
        .post(app.entry_url(&entry_id.to_string(), "/recover"))
        .bearer_auth(&app.token)
        .json(&json!({ "ended_at": declared }))
        .send()
        .await
        .expect("the api answers the recover call")
        .text()
        .await
        .expect("the recover answer has a body");
    let body: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("the recover answer is json: {e}: {raw}"));

    assert_eq!(
        body["data"]["worked_minutes"],
        json!(420),
        "seven hours, as declared, not thirty: {body}"
    );

    // And now the day can start.
    let started = client
        .post(app.url("/time-entries"))
        .bearer_auth(&app.token)
        .json(&json!({ "task_id": app.task_id }))
        .send()
        .await
        .expect("the api answers the start call");
    assert!(
        started.status().is_success(),
        "the worker is unblocked once yesterday is settled: {}",
        started.status()
    );

    app.cleanup().await;
}
