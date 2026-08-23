//! The worker's half of the correction loop, end to end: file, amend,
//! withdraw, list — and the cross-tenant case the bare-id routes
//! (`PATCH`/`DELETE /field/assignment-reports/{id}`) exist to guard against,
//! since they carry no organization in the path.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{harness, issuer};

/// A worker files a report, amends it while it is pending, and sees the
/// amendment reflected in their own list — resolved reports would show up
/// there too, but this one never gets resolved in this test.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_worker_files_and_amends_a_report_then_sees_it_in_their_own_list() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let filed = client
        .post(app.report_assignment_url(app.task_assignment_id))
        .bearer_auth(&app.token)
        .json(&json!({ "reported_minutes": 300, "comment": "Plus long que prévu" }))
        .send()
        .await
        .expect("the api answers the report call");
    let status = filed.status();
    let raw = filed.text().await.expect("the report answer has a body");
    assert!(status.is_success(), "filing failed with {status}: {raw}");
    let filed: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("the report answer is json: {e}: {raw}"));
    assert_eq!(filed["data"]["reported_minutes"], json!(300), "{filed}");
    assert_eq!(filed["data"]["resolution"], json!("PENDING"), "{filed}");
    let report_id = filed["data"]["id"]
        .as_str()
        .expect("a report id")
        .to_owned();

    let amended = client
        .patch(app.assignment_report_url(&report_id))
        .bearer_auth(&app.token)
        .json(&json!({ "reported_minutes": 240, "comment": null }))
        .send()
        .await
        .expect("the api answers the amend call");
    assert!(amended.status().is_success(), "{}", amended.status());
    let amended: serde_json::Value = amended.json().await.expect("the amend answer is json");
    assert_eq!(amended["data"]["reported_minutes"], json!(240), "{amended}");

    let listed: serde_json::Value = client
        .get(app.url("/assignment-reports"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");
    let items = listed["data"]
        .as_array()
        .expect("the list carries an array");
    let mine = items
        .iter()
        .find(|item| item["id"] == json!(report_id))
        .unwrap_or_else(|| {
            panic!("the amended report is missing from the caller's own list: {listed}")
        });
    assert_eq!(mine["reported_minutes"], json!(240), "{mine}");

    app.cleanup().await;
}

/// Withdrawing a pending report removes it — nothing left to amend, nothing
/// left in the caller's list.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_worker_withdraws_a_pending_report() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let filed: serde_json::Value = client
        .post(app.report_assignment_url(app.task_assignment_id))
        .bearer_auth(&app.token)
        .json(&json!({ "reported_minutes": 120, "comment": null }))
        .send()
        .await
        .expect("the api answers the report call")
        .json()
        .await
        .expect("the report answer is json");
    let report_id = filed["data"]["id"]
        .as_str()
        .expect("a report id")
        .to_owned();

    let withdrawn = client
        .delete(app.assignment_report_url(&report_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the withdraw call");
    assert_eq!(withdrawn.status(), 204, "{}", withdrawn.status());

    let listed: serde_json::Value = client
        .get(app.url("/assignment-reports"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");
    let items = listed["data"]
        .as_array()
        .expect("the list carries an array");
    assert!(
        !items.iter().any(|item| item["id"] == json!(report_id)),
        "a withdrawn report must not appear in the caller's list: {listed}"
    );

    app.cleanup().await;
}

/// The security rule `AssignmentReportService::report_assignment` exists to
/// enforce: reporting on a colleague's assignment is refused, not silently
/// filed under the caller's own name.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn reporting_on_a_colleagues_assignment_is_forbidden() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let response = client
        .post(app.report_assignment_url(app.other_task_assignment_id))
        .bearer_auth(&app.token)
        .json(&json!({ "reported_minutes": 60, "comment": null }))
        .send()
        .await
        .expect("the api answers the report call");
    assert_eq!(
        response.status(),
        403,
        "reporting on someone else's assignment must be refused"
    );

    app.cleanup().await;
}

/// The cross-tenant case `PATCH`/`DELETE /field/assignment-reports/{id}`
/// exist to guard against: those routes carry no organization in the path,
/// so a member of a *different* organization must not be able to reach a
/// report that belongs to someone else's — the handler has to derive the
/// organization from the row itself, never trust one supplied by the
/// caller.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_of_a_different_organization_cannot_amend_the_report() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let filed: serde_json::Value = client
        .post(app.report_assignment_url(app.task_assignment_id))
        .bearer_auth(&app.token)
        .json(&json!({ "reported_minutes": 90, "comment": null }))
        .send()
        .await
        .expect("the api answers the report call")
        .json()
        .await
        .expect("the report answer is json");
    let report_id = filed["data"]["id"]
        .as_str()
        .expect("a report id")
        .to_owned();

    let intruder = seed_intruder_in_another_organization(&app.pool).await;

    let attempt = client
        .patch(app.assignment_report_url(&report_id))
        .bearer_auth(&intruder.token)
        .json(&json!({ "reported_minutes": 1, "comment": null }))
        .send()
        .await
        .expect("the api answers the amend call");
    assert_eq!(
        attempt.status(),
        403,
        "a member of another organization must not reach this report"
    );

    // Untouched: the intruder's rejected attempt must not have mutated it.
    let listed: serde_json::Value = client
        .get(app.url("/assignment-reports"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");
    let mine = listed["data"]
        .as_array()
        .expect("the list carries an array")
        .iter()
        .find(|item| item["id"] == json!(report_id))
        .expect("the report must still be there");
    assert_eq!(mine["reported_minutes"], json!(90), "{mine}");

    app.cleanup().await;
    intruder.cleanup(&app.pool).await;
}

struct Intruder {
    token: String,
    organization_id: Uuid,
    user_id: Uuid,
}

impl Intruder {
    async fn cleanup(&self, pool: &PgPool) {
        for statement in [
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

/// A user, a fresh organization of their own, and a membership in it — just
/// enough for the auth middleware and `resolve_field_actor` to resolve a
/// real identity that belongs to *no* organization the fixture's own report
/// lives in.
async fn seed_intruder_in_another_organization(pool: &PgPool) -> Intruder {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-intruder-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("intruder-{user_id}@example.com"))
    .bind(format!("intruder-{user_id}"))
    .bind("Intruder")
    .bind(&sub)
    .execute(pool)
    .await
    .expect("seed the intruder user");

    let organization_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id)
        .bind("Intruder Org")
        .bind(format!("intruder-org-{organization_id}"))
        .bind(user_id)
        .execute(pool)
        .await
        .expect("seed the intruder's own organization");

    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(user_id)
    .bind("Intruder")
    .execute(pool)
    .await
    .expect("seed the intruder's own membership");

    Intruder {
        token: issuer::mint(&sub),
        organization_id,
        user_id,
    }
}
