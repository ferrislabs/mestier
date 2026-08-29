//! #309: the work-time routes are addressed by bare `member_id`, with no
//! organization in the path at all. `require_member_target` (see
//! `handlers_planning::require_member_target`) loads the target member's
//! seat first and only then checks the *caller's own* membership against
//! *that seat's* organization — membership is the outer gate, checked
//! before any permission is consulted. This suite proves that gate holds
//! over a real HTTP call for all three routes, and that the fix does not
//! also lock out the legitimate, same-organization caller.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{harness, issuer};

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn get_work_time_refuses_a_caller_outside_the_members_organization() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let intruder = seed_intruder_in_another_organization(&app.pool).await;

    let refused = client
        .get(app.work_time_url(app.assignee_member_id, "?from=2026-08-01&to=2026-08-31"))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("the api answers the get call for the intruder");
    assert_eq!(
        refused.status(),
        403,
        "a caller outside the member's organization must be refused, not 404 or 200"
    );

    // Regression guard: the owning organization's own caller must still work.
    let allowed = client
        .get(app.work_time_url(app.assignee_member_id, "?from=2026-08-01&to=2026-08-31"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call for the owning organization");
    assert_eq!(
        allowed.status(),
        200,
        "the owning organization's own caller must keep working"
    );

    intruder.cleanup(&app.pool).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn put_rhythm_refuses_a_caller_outside_the_members_organization() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let intruder = seed_intruder_in_another_organization(&app.pool).await;

    let payload = json!({
        "effective_from": "2026-01-01",
        "effective_to": null,
        "slots": [
            { "weekday": 1, "starts_minute": 480, "ends_minute": 720 },
        ],
    });

    let refused = client
        .put(app.rhythm_url(app.assignee_member_id))
        .bearer_auth(&intruder.token)
        .json(&payload)
        .send()
        .await
        .expect("the api answers the put call for the intruder");
    assert_eq!(
        refused.status(),
        403,
        "a caller outside the member's organization must be refused, not 404 or 200"
    );

    // Regression guard: the owning organization's own caller must still work.
    let allowed = client
        .put(app.rhythm_url(app.assignee_member_id))
        .bearer_auth(&app.token)
        .json(&payload)
        .send()
        .await
        .expect("the api answers the put call for the owning organization");
    let status = allowed.status();
    let body: serde_json::Value = allowed
        .json()
        .await
        .unwrap_or_else(|e| panic!("the put answer is json: {e}"));
    assert!(
        status.is_success(),
        "the owning organization's own caller must keep working: {status}: {body}"
    );

    intruder.cleanup(&app.pool).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn put_work_slots_refuses_a_caller_outside_the_members_organization() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let intruder = seed_intruder_in_another_organization(&app.pool).await;

    let payload = json!({ "slots": [] });

    let refused = client
        .put(app.work_slots_url(app.assignee_member_id, "?from=2026-08-01&to=2026-08-31"))
        .bearer_auth(&intruder.token)
        .json(&payload)
        .send()
        .await
        .expect("the api answers the put call for the intruder");
    assert_eq!(
        refused.status(),
        403,
        "a caller outside the member's organization must be refused, not 404 or 200"
    );

    // Regression guard: the owning organization's own caller must still work.
    let allowed = client
        .put(app.work_slots_url(app.assignee_member_id, "?from=2026-08-01&to=2026-08-31"))
        .bearer_auth(&app.token)
        .json(&payload)
        .send()
        .await
        .expect("the api answers the put call for the owning organization");
    let status = allowed.status();
    let body: serde_json::Value = allowed
        .json()
        .await
        .unwrap_or_else(|e| panic!("the put answer is json: {e}"));
    assert!(
        status.is_success(),
        "the owning organization's own caller must keep working: {status}: {body}"
    );

    intruder.cleanup(&app.pool).await;
    app.cleanup().await;
}

/// Mirrors `libs/handlers-field/tests/http_e2e/assignment_reports.rs`'s own
/// `Intruder`: a throwaway second organization and member, seeded directly
/// via SQL, whose token is real (signed by the same fake issuer) but whose
/// membership belongs to none of the fixture's own organization.
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
/// enough for the auth middleware and `require_org_membership` to resolve a
/// real identity that belongs to *no* organization the fixture's own member
/// lives in.
async fn seed_intruder_in_another_organization(pool: &PgPool) -> Intruder {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-planning-intruder-{user_id}");
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
