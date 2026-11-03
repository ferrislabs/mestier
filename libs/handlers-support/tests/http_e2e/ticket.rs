use mestier_core::TicketStatus;
use reqwest::{Client, Response, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness;

async fn json_body(response: Response) -> Value {
    response.json().await.unwrap()
}

fn subjects(body: &Value) -> Vec<&str> {
    body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|ticket| ticket["subject"].as_str().unwrap())
        .collect()
}

fn numbers(body: &Value) -> Vec<i64> {
    body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|ticket| ticket["number"].as_i64().unwrap())
        .collect()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn member_without_view_support_is_refused_on_list() {
    let app = harness::start().await;
    let client = Client::new();

    let anonymous = client.get(app.tickets_url("")).send().await.unwrap();
    let outsider = client
        .get(app.tickets_url(""))
        .bearer_auth(&app.outsider.token)
        .send()
        .await
        .unwrap();

    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(outsider.status(), StatusCode::FORBIDDEN);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn member_without_view_support_is_refused_on_get() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let response = client
        .get(app.ticket_url(ticket.id.0, ""))
        .bearer_auth(&app.outsider.token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn viewer_reads_the_ticket_with_every_field() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(
            app.organization_id,
            "Printer on fire",
            json!({ "team": "tech" }),
        )
        .await;

    let response = client
        .get(app.ticket_url(ticket.id.0, ""))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let data = &body["data"];
    assert_eq!(data["id"], ticket.id.0.to_string());
    assert_eq!(data["number"], 1);
    assert_eq!(data["subject"], "Printer on fire");
    assert_eq!(data["body"], "Printer on fire body");
    assert_eq!(data["status"], "new");
    assert_eq!(data["assignee_member_id"], Value::Null);
    assert_eq!(data["assigned_at"], Value::Null);
    assert_eq!(data["custom_fields"], json!({ "team": "tech" }));
    assert_eq!(data["badges"], json!([]));
    assert_eq!(data["external_id"], Value::Null);
    assert!(data["created_at"].is_string());
    assert!(data["updated_at"].is_string());
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn view_without_manage_is_refused_on_transition() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let response = client
        .patch(app.ticket_url(ticket.id.0, "/status"))
        .bearer_auth(&app.viewer.token)
        .json(&json!({ "status": "in_progress" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let unchanged = client
        .get(app.ticket_url(ticket.id.0, ""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(json_body(unchanged).await["data"]["status"], "new");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn transition_returns_the_new_status() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let response = client
        .patch(app.ticket_url(ticket.id.0, "/status"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "status": "waiting_customer" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["data"]["id"], ticket.id.0.to_string());
    assert_eq!(body["data"]["status"], "waiting_customer");
    assert_eq!(body["data"]["subject"], "Printer on fire");
    let reread = client
        .get(app.ticket_url(ticket.id.0, ""))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        json_body(reread).await["data"]["status"],
        "waiting_customer"
    );
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn unknown_status_is_422() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let transition = client
        .patch(app.ticket_url(ticket.id.0, "/status"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "status": "closed" }))
        .send()
        .await
        .unwrap();
    let list = client
        .get(app.tickets_url("?status=new,closed"))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();

    assert_eq!(transition.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json_body(transition).await["code"],
        "E_UNPROCESSABLE_ENTITY"
    );
    assert_eq!(list.status(), StatusCode::UNPROCESSABLE_ENTITY);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn list_filters_by_custom_field() {
    let app = harness::start().await;
    let client = Client::new();
    app.seed_ticket(app.organization_id, "Tech one", json!({ "team": "tech" }))
        .await;
    app.seed_ticket(
        app.organization_id,
        "Billing one",
        json!({ "team": "billing" }),
    )
    .await;
    app.seed_ticket(
        app.organization_id,
        "Tech two",
        json!({ "team": "tech", "seats": 12 }),
    )
    .await;
    app.seed_ticket(app.organization_id, "Bare", json!({}))
        .await;

    let response = client
        .get(app.tickets_url("?custom_field_key=team&custom_field_value=tech"))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(subjects(&body), ["Tech two", "Tech one"]);
    assert_eq!(body["pagination"]["total"], 2);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn list_with_only_custom_field_key_is_422() {
    let app = harness::start().await;
    let client = Client::new();

    let key_only = client
        .get(app.tickets_url("?custom_field_key=team"))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    let value_only = client
        .get(app.tickets_url("?custom_field_value=tech"))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();

    assert_eq!(key_only.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json_body(key_only).await["code"], "E_UNPROCESSABLE_ENTITY");
    assert_eq!(value_only.status(), StatusCode::UNPROCESSABLE_ENTITY);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn list_filters_by_status() {
    let app = harness::start().await;
    let client = Client::new();
    app.seed_ticket(app.organization_id, "Fresh", json!({}))
        .await;
    let working = app
        .seed_ticket(app.organization_id, "Working", json!({}))
        .await;
    let done = app
        .seed_ticket(app.organization_id, "Done", json!({}))
        .await;
    app.set_status(&working, TicketStatus::InProgress).await;
    app.set_status(&done, TicketStatus::Resolved).await;

    let open = client
        .get(app.tickets_url("?status=new,in_progress"))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();
    let resolved = client
        .get(app.tickets_url("?status=resolved"))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();
    let everything = client
        .get(app.tickets_url(""))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();

    assert_eq!(open.status(), StatusCode::OK);
    let open = json_body(open).await;
    assert_eq!(subjects(&open), ["Working", "Fresh"]);
    assert_eq!(open["pagination"]["total"], 2);
    let resolved = json_body(resolved).await;
    assert_eq!(subjects(&resolved), ["Done"]);
    assert_eq!(resolved["data"][0]["status"], "resolved");
    assert_eq!(json_body(everything).await["pagination"]["total"], 3);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn list_filters_by_assignee_and_badge() {
    let app = harness::start().await;
    let client = Client::new();
    let mine = app
        .seed_ticket(app.organization_id, "Mine", json!({}))
        .await;
    let tagged = app
        .seed_ticket(app.organization_id, "Tagged", json!({}))
        .await;
    app.seed_ticket(app.organization_id, "Plain", json!({}))
        .await;

    let assigned = client
        .put(app.ticket_url(mine.id.0, "/assignee"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "member_id": app.admin.member_id }))
        .send()
        .await
        .unwrap();
    let badged = client
        .post(app.ticket_url(tagged.id.0, "/badges"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "label": "VIP", "tone": "brand" }))
        .send()
        .await
        .unwrap();
    assert_eq!(assigned.status(), StatusCode::OK);
    assert_eq!(badged.status(), StatusCode::OK);

    let by_assignee = client
        .get(app.tickets_url(&format!("?assignee={}", app.admin.member_id)))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();
    let by_badge = client
        .get(app.tickets_url("?badge=VIP"))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();

    assert_eq!(subjects(&json_body(by_assignee).await), ["Mine"]);
    assert_eq!(subjects(&json_body(by_badge).await), ["Tagged"]);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn ticket_of_another_organization_is_404() {
    let app = harness::start().await;
    let client = Client::new();
    let foreign = app
        .seed_ticket(app.other_organization_id, "Not yours", json!({}))
        .await;
    let id = foreign.id.0;

    let get = client
        .get(app.ticket_url(id, ""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    let transition = client
        .patch(app.ticket_url(id, "/status"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "status": "resolved" }))
        .send()
        .await
        .unwrap();
    let assign = client
        .put(app.ticket_url(id, "/assignee"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "member_id": app.admin.member_id }))
        .send()
        .await
        .unwrap();
    let badge = client
        .post(app.ticket_url(id, "/badges"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "label": "VIP" }))
        .send()
        .await
        .unwrap();
    let unbadge = client
        .delete(app.ticket_url(id, "/badges/VIP"))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    let unknown = client
        .get(app.ticket_url(Uuid::now_v7(), ""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();

    for response in [get, transition, assign, badge, unbadge, unknown] {
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    let listed = client
        .get(app.tickets_url(""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(json_body(listed).await["pagination"]["total"], 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn assigning_a_member_of_another_organization_is_404() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let response = client
        .put(app.ticket_url(ticket.id.0, "/assignee"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "member_id": app.other_member_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let unchanged = client
        .get(app.ticket_url(ticket.id.0, ""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        json_body(unchanged).await["data"]["assignee_member_id"],
        Value::Null
    );
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn assignee_is_set_then_cleared() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let assigned = client
        .put(app.ticket_url(ticket.id.0, "/assignee"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "member_id": app.viewer.member_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(assigned.status(), StatusCode::OK);
    let assigned = json_body(assigned).await;
    assert_eq!(
        assigned["data"]["assignee_member_id"],
        app.viewer.member_id.to_string()
    );
    assert!(assigned["data"]["assigned_at"].is_string());

    let refused = client
        .put(app.ticket_url(ticket.id.0, "/assignee"))
        .bearer_auth(&app.viewer.token)
        .json(&json!({ "member_id": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), StatusCode::FORBIDDEN);

    let cleared = client
        .put(app.ticket_url(ticket.id.0, "/assignee"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "member_id": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(cleared.status(), StatusCode::OK);
    let cleared = json_body(cleared).await;
    assert_eq!(cleared["data"]["assignee_member_id"], Value::Null);
    assert_eq!(cleared["data"]["assigned_at"], Value::Null);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn badges_are_added_then_removed() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let first = client
        .post(app.ticket_url(ticket.id.0, "/badges"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "label": "High", "tone": "error" }))
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(
        json_body(first).await["data"]["badges"],
        json!([{ "label": "High", "tone": "error" }])
    );

    let second = client
        .post(app.ticket_url(ticket.id.0, "/badges"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "label": "To call" }))
        .send()
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(
        json_body(second).await["data"]["badges"],
        json!([
            { "label": "High", "tone": "error" },
            { "label": "To call", "tone": "neutral" }
        ])
    );

    let retoned = client
        .post(app.ticket_url(ticket.id.0, "/badges"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "label": "High", "tone": "warning" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        json_body(retoned).await["data"]["badges"][0],
        json!({ "label": "High", "tone": "warning" })
    );

    let refused = client
        .delete(app.ticket_url(ticket.id.0, "/badges/High"))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), StatusCode::FORBIDDEN);

    let removed = client
        .delete(app.ticket_url(ticket.id.0, "/badges/To%20call"))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(removed.status(), StatusCode::OK);
    assert_eq!(
        json_body(removed).await["data"]["badges"],
        json!([{ "label": "High", "tone": "warning" }])
    );

    let absent = client
        .delete(app.ticket_url(ticket.id.0, "/badges/Nope"))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(absent.status(), StatusCode::OK);
    assert_eq!(
        json_body(absent).await["data"]["badges"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let unknown_tone = client
        .post(app.ticket_url(ticket.id.0, "/badges"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "label": "Odd", "tone": "purple" }))
        .send()
        .await
        .unwrap();
    assert_eq!(unknown_tone.status(), StatusCode::UNPROCESSABLE_ENTITY);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn list_paginates_with_metadata() {
    let app = harness::start().await;
    let client = Client::new();
    for index in 1..=5 {
        app.seed_ticket(app.organization_id, &format!("Ticket {index}"), json!({}))
            .await;
    }

    let second = client
        .get(app.tickets_url("?per_page=2&page=2"))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();
    let last = client
        .get(app.tickets_url("?per_page=2&page=3"))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();

    assert_eq!(second.status(), StatusCode::OK);
    let second = json_body(second).await;
    assert_eq!(numbers(&second), [3, 2]);
    assert_eq!(subjects(&second), ["Ticket 3", "Ticket 2"]);
    let meta = &second["pagination"];
    assert_eq!(meta["per_page"], 2);
    assert_eq!(meta["current_page"], 2);
    assert_eq!(meta["first_page"], 1);
    assert_eq!(meta["total"], 5);
    assert_eq!(meta["last_page"], 3);
    assert_eq!(meta["next_page"], 3);
    assert_eq!(meta["prev_page"], 1);
    assert_eq!(meta["is_empty"], false);

    let last = json_body(last).await;
    assert_eq!(numbers(&last), [1]);
    assert_eq!(last["pagination"]["next_page"], Value::Null);
    assert_eq!(last["pagination"]["prev_page"], 2);
    app.cleanup().await;
}
