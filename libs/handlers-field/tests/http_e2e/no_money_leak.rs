//! #309, part 2: "a field worker never receives money".
//!
//! Unconditional, and asserted against every route this crate serves, not
//! just the ones that look financial — so it holds regardless of what bits
//! the worker's role happens to carry, and it is the assertion that catches
//! a response field added later without anyone thinking about who reads it.
//! Response bodies are walked as raw `serde_json::Value`, not deserialized
//! into the typed DTOs from `response.rs`: a typed struct only ever shows the
//! fields this test already knows about, while a `Value` shows whatever the
//! server actually sent, including a field nobody remembered to add here.

use serde_json::json;

use crate::harness;

/// Fails loudly, naming the offending key and its full path (e.g.
/// `data.photos.0.rate_cents`), if any object key at any nesting depth looks
/// like it carries a cost, a rate, an amount in cents, or a salary.
fn assert_no_money_leak(value: &serde_json::Value) {
    const FORBIDDEN: [&str; 4] = ["cost", "rate", "cents", "salary"];

    fn walk(value: &serde_json::Value, path: &str) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, child) in map {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    let lower = key.to_lowercase();
                    if let Some(hit) = FORBIDDEN.iter().find(|needle| lower.contains(**needle)) {
                        panic!(
                            "money leak: key `{key}` at `{child_path}` looks like it carries \
                             a `{hit}` — a field worker must never receive this in a response"
                        );
                    }
                    walk(child, &child_path);
                }
            }
            serde_json::Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    let child_path = if path.is_empty() {
                        index.to_string()
                    } else {
                        format!("{path}.{index}")
                    };
                    walk(item, &child_path);
                }
            }
            _ => {}
        }
    }

    walk(value, "");
}

/// Drives all 12 routes `handlers-field` serves with a real field-worker
/// token against a real seeded fixture, and checks every response body for a
/// leak. Passing today is expected — the DTOs carry no such field — this is
/// a regression guard, not a bug-fix vehicle.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_field_worker_never_receives_money() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    // 1. GET .../field/tasks
    let tasks: serde_json::Value = client
        .get(app.url("/tasks"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the task list")
        .json()
        .await
        .expect("the task list is json");
    assert_no_money_leak(&tasks);

    // 2. GET .../field/current
    let current: serde_json::Value = client
        .get(app.url("/current"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers /current")
        .json()
        .await
        .expect("/current is json");
    assert_no_money_leak(&current);

    // 3. POST .../field/time-entries — clock on to the caller's own job.
    let started: serde_json::Value = client
        .post(app.url("/time-entries"))
        .bearer_auth(&app.token)
        .json(&json!({ "task_id": app.task_id }))
        .send()
        .await
        .expect("the api answers the start call")
        .json()
        .await
        .expect("the start answer is json");
    assert_no_money_leak(&started);
    let entry_id = started["data"]["id"]
        .as_str()
        .expect("an entry id")
        .to_owned();

    // 4. POST .../field/time-entries/declare — a stretch well before the
    // entry just started, so it cannot overlap the job still running.
    let now = harness::now_storable();
    let declared: serde_json::Value = client
        .post(app.url("/time-entries/declare"))
        .bearer_auth(&app.token)
        .json(&json!({
            "task_id": app.task_id,
            "started_at": (now - chrono::Duration::hours(10)).to_rfc3339(),
            "ended_at": (now - chrono::Duration::hours(9)).to_rfc3339(),
        }))
        .send()
        .await
        .expect("the api answers the declare call")
        .json()
        .await
        .expect("the declare answer is json");
    assert_no_money_leak(&declared);

    // 5. POST /field/time-entries/{id}/stop — close the job started above.
    let stopped: serde_json::Value = client
        .post(app.entry_url(&entry_id, "/stop"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the stop call")
        .json()
        .await
        .expect("the stop answer is json");
    assert_no_money_leak(&stopped);

    // 6. POST /field/time-entries/{id}/recover — a forgotten stretch, seeded
    // directly: the only way one exists is by never having been closed live.
    let forgotten_id = harness::seed_forgotten_entry(&app.pool, &app).await;
    let recover_ended_at = (harness::now_storable() - chrono::Duration::hours(23)).to_rfc3339();
    let recovered: serde_json::Value = client
        .post(app.entry_url(&forgotten_id.to_string(), "/recover"))
        .bearer_auth(&app.token)
        .json(&json!({ "ended_at": recover_ended_at }))
        .send()
        .await
        .expect("the api answers the recover call")
        .json()
        .await
        .expect("the recover answer is json");
    assert_no_money_leak(&recovered);

    // 7. POST /field/time-entries/{id}/photos — allowed after the entry is
    // closed, which the one from step 5 already is.
    let photo: serde_json::Value = client
        .post(app.entry_url(&entry_id, "/photos"))
        .bearer_auth(&app.token)
        .json(&json!({ "phase": "BEFORE", "storage_key": "uploads/field/before.jpg" }))
        .send()
        .await
        .expect("the api answers the photo call")
        .json()
        .await
        .expect("the photo answer is json");
    assert_no_money_leak(&photo);

    // 8. POST .../field/day-end — nothing left running, so this only writes
    // the day log.
    let ended: serde_json::Value = client
        .post(app.url("/day-end"))
        .bearer_auth(&app.token)
        .json(&json!({}))
        .send()
        .await
        .expect("the api answers the day-end call")
        .json()
        .await
        .expect("the day-end answer is json");
    assert_no_money_leak(&ended);

    // 9. POST .../field/assignments/{id}/report
    let filed: serde_json::Value = client
        .post(app.report_assignment_url(app.task_assignment_id))
        .bearer_auth(&app.token)
        .json(&json!({ "reported_minutes": 90, "comment": "RAS" }))
        .send()
        .await
        .expect("the api answers the report call")
        .json()
        .await
        .expect("the report answer is json");
    assert_no_money_leak(&filed);
    let report_id = filed["data"]["id"]
        .as_str()
        .expect("a report id")
        .to_owned();

    // 10. PATCH /field/assignment-reports/{id}
    let amended: serde_json::Value = client
        .patch(app.assignment_report_url(&report_id))
        .bearer_auth(&app.token)
        .json(&json!({ "reported_minutes": 80, "comment": null }))
        .send()
        .await
        .expect("the api answers the amend call")
        .json()
        .await
        .expect("the amend answer is json");
    assert_no_money_leak(&amended);

    // 11. GET .../field/assignment-reports
    let listed: serde_json::Value = client
        .get(app.url("/assignment-reports"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");
    assert_no_money_leak(&listed);

    // 12. DELETE /field/assignment-reports/{id} — 204 No Content, so there
    // is no body to walk: `Response::NoContent` serializes nothing, meaning
    // there is nothing here that could carry a leak in the first place.
    let withdrawn = client
        .delete(app.assignment_report_url(&report_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the withdraw call");
    assert_eq!(withdrawn.status(), 204, "{}", withdrawn.status());

    app.cleanup().await;
}
