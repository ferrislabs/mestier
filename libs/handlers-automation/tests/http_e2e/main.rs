//! End-to-end tests for the automation permission gate (#493).
//!
//! Same shape as `handlers-invoice`'s own suite: a request enters over a real
//! socket and crosses the whole stack — typed routing, rate-limit and auth
//! middleware with a real RS256 token checked against a real JWKS fetch, the
//! handler, the use case, a Postgres transaction. Nothing in that chain is
//! doubled, which is also why these are `#[ignore]`d.
//!
//! ```bash
//! docker compose up -d postgres redis
//! source .env
//! cargo test -p handlers-automation --test http_e2e -- --ignored
//! ```

mod harness;
mod issuer;

use serde_json::{Value, json};

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn create_workflow(app: &harness::App, token: &str) -> reqwest::Response {
    client()
        .post(app.workflows_url())
        .bearer_auth(token)
        .json(&json!({
            "name": "Test workflow",
            "description": null
        }))
        .send()
        .await
        .expect("the api answers the create call")
}

/// #493: a member with no role at all — bare membership, the gate this
/// issue closes — is refused reading the organization's workflows, not
/// served a silent empty list.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_view_automation_is_refused_the_workflow_list() {
    let app = harness::start().await;

    let response = client()
        .get(app.workflows_url())
        .bearer_auth(&app.no_role_token)
        .send()
        .await
        .expect("the api answers the no-role member's list call");

    assert_eq!(
        response.status(),
        403,
        "VIEW_AUTOMATION must gate the list, not just organization membership"
    );

    app.cleanup().await;
}

/// #493: the same no-role member is refused creating a workflow —
/// `MANAGE_AUTOMATION` gates the write side the way `VIEW_AUTOMATION` gates
/// reads above.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_manage_automation_is_refused_creating_a_workflow() {
    let app = harness::start().await;

    let response = create_workflow(&app, &app.no_role_token).await;

    assert_eq!(
        response.status(),
        403,
        "MANAGE_AUTOMATION must gate creation, not just organization membership"
    );

    app.cleanup().await;
}

/// A member holding `VIEW_AUTOMATION` alone reads the list, but is refused
/// creating a workflow — the two bits are independent, `VIEW_AUTOMATION`
/// does not imply `MANAGE_AUTOMATION`.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_view_only_member_can_list_but_not_create() {
    let app = harness::start().await;

    let list_response = client()
        .get(app.workflows_url())
        .bearer_auth(&app.view_only_token)
        .send()
        .await
        .expect("the api answers the view-only member's list call");
    assert_eq!(list_response.status(), 200);

    let create_response = create_workflow(&app, &app.view_only_token).await;
    assert_eq!(
        create_response.status(),
        403,
        "VIEW_AUTOMATION must not also grant MANAGE_AUTOMATION"
    );

    app.cleanup().await;
}

/// A member holding both bits gets through on both the read and the write.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_with_both_bits_can_list_and_create() {
    let app = harness::start().await;

    let list_response = client()
        .get(app.workflows_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the full-access member's list call");
    assert_eq!(list_response.status(), 200);

    let create_response = create_workflow(&app, &app.token).await;
    assert_eq!(create_response.status(), 201);

    app.cleanup().await;
}

/// A caller who belongs to a *different* organization, and holds both
/// automation bits there, is still refused here — membership in the
/// organization actually named in the path is not bypassed by holding the
/// bit somewhere else.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_outsider_holding_both_bits_elsewhere_is_still_refused() {
    let app = harness::start().await;

    let response = client()
        .get(app.workflows_url())
        .bearer_auth(&app.outsider_token)
        .send()
        .await
        .expect("the api answers the outsider's list call");

    assert_eq!(
        response.status(),
        403,
        "membership in this organization must not be bypassed by holding the bit in another one"
    );

    app.cleanup().await;
}

/// `settings.rs` holds both a read and a write handler behind one file —
/// this proves each is gated on its own bit, not the file as a whole.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_view_only_member_reads_settings_but_cannot_update_them() {
    let app = harness::start().await;

    let get_response = client()
        .get(app.settings_url())
        .bearer_auth(&app.view_only_token)
        .send()
        .await
        .expect("the api answers the view-only member's settings read");
    assert_eq!(get_response.status(), 200);

    let put_response = client()
        .put(app.settings_url())
        .bearer_auth(&app.view_only_token)
        .json(&json!({
            "event_retention_seconds": 7_776_000_u64,
            "succeeded_run_retention_seconds": 2_592_000_u64,
            "retry_schedule_seconds": [5, 30, 120, 600, 3600, 21600],
            "disable_target_after": 20
        }))
        .send()
        .await
        .expect("the api answers the view-only member's settings write");
    assert_eq!(
        put_response.status(),
        403,
        "MANAGE_AUTOMATION must gate the settings write, VIEW_AUTOMATION must not"
    );

    app.cleanup().await;
}

/// `workflow/trigger.rs` holds both a read and a write handler behind one
/// file too — same split proof as settings, on a resource loaded through
/// `workflow::find_workflow_in_org` rather than checked directly off the
/// path.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_view_only_member_reads_a_trigger_but_cannot_set_it() {
    let app = harness::start().await;
    let created = create_workflow(&app, &app.token).await;
    let workflow: Value = created.json().await.expect("the create answer is json");
    let workflow_id = workflow["data"]["id"]
        .as_str()
        .expect("the created workflow carries an id");

    let get_response = client()
        .get(app.workflow_trigger_url(workflow_id))
        .bearer_auth(&app.view_only_token)
        .send()
        .await
        .expect("the api answers the view-only member's trigger read");
    assert_eq!(get_response.status(), 200);

    let put_response = client()
        .put(app.workflow_trigger_url(workflow_id))
        .bearer_auth(&app.view_only_token)
        .json(&json!({ "mode": "events", "event_names": [] }))
        .send()
        .await
        .expect("the api answers the view-only member's trigger write");
    assert_eq!(
        put_response.status(),
        403,
        "MANAGE_AUTOMATION must gate the trigger write, VIEW_AUTOMATION must not"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_valid_expression_resolves_against_the_supplied_context() {
    let app = harness::start().await;

    let response = client()
        .post(app.evaluate_expression_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "template": "{{ trigger.name }}",
            "context": {
                "trigger": { "name": "Brioche" },
                "connectors": {},
                "loop": null
            }
        }))
        .send()
        .await
        .expect("the api answers the evaluate call");

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.expect("the answer is json");
    assert_eq!(body["data"]["value"], json!("Brioche"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_missing_path_is_refused_naming_the_path() {
    let app = harness::start().await;

    let response = client()
        .post(app.evaluate_expression_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "template": "{{ trigger.missing }}",
            "context": { "trigger": {}, "connectors": {}, "loop": null }
        }))
        .send()
        .await
        .expect("the api answers the evaluate call");

    assert_eq!(response.status(), 422);
    let body: Value = response.json().await.expect("the answer is json");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("trigger.missing"),
        "{body}"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn loop_outside_a_loop_frame_is_refused() {
    let app = harness::start().await;

    let response = client()
        .post(app.evaluate_expression_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "template": "{{ loop.item }}",
            "context": { "trigger": null, "connectors": {}, "loop": null }
        }))
        .send()
        .await
        .expect("the api answers the evaluate call");

    assert_eq!(response.status(), 422);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_oversized_context_is_refused() {
    let app = harness::start().await;

    let mut connectors = serde_json::Map::new();
    for index in 0..300 {
        connectors.insert(format!("c{index}"), json!(true));
    }

    let response = client()
        .post(app.evaluate_expression_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "template": "{{ 1 }}",
            "context": { "trigger": null, "connectors": connectors, "loop": null }
        }))
        .send()
        .await
        .expect("the api answers the evaluate call");

    assert_eq!(response.status(), 422);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_view_only_member_can_evaluate_an_expression() {
    let app = harness::start().await;

    let response = client()
        .post(app.evaluate_expression_url())
        .bearer_auth(&app.view_only_token)
        .json(&json!({
            "template": "{{ 1 }}",
            "context": { "trigger": null, "connectors": {}, "loop": null }
        }))
        .send()
        .await
        .expect("the api answers the evaluate call");

    assert_eq!(response.status(), 200);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_caller_without_view_automation_is_refused_evaluating_an_expression() {
    let app = harness::start().await;

    let response = client()
        .post(app.evaluate_expression_url())
        .bearer_auth(&app.no_role_token)
        .json(&json!({
            "template": "{{ 1 }}",
            "context": { "trigger": null, "connectors": {}, "loop": null }
        }))
        .send()
        .await
        .expect("the api answers the evaluate call");

    assert_eq!(
        response.status(),
        403,
        "VIEW_AUTOMATION must gate expression evaluation too"
    );

    app.cleanup().await;
}
