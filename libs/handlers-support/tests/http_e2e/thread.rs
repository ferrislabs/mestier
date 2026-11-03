use mestier_core::EntryKind;
use reqwest::{Client, Response, StatusCode, header::CONTENT_TYPE};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App};

async fn json_body(response: Response) -> Value {
    response.json().await.unwrap()
}

fn kinds(body: &Value) -> Vec<&str> {
    body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["kind"].as_str().unwrap())
        .collect()
}

async fn post_entry(
    app: &App,
    client: &Client,
    token: &str,
    ticket_id: Uuid,
    kind: &str,
    body: &str,
) -> Response {
    client
        .post(app.entries_url(ticket_id))
        .bearer_auth(token)
        .json(&json!({ "kind": kind, "body": body }))
        .send()
        .await
        .unwrap()
}

async fn list_entries(app: &App, client: &Client, token: &str, ticket_id: Uuid) -> Value {
    let response = client
        .get(app.entries_url(ticket_id))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    json_body(response).await
}

async fn upload(
    app: &App,
    client: &Client,
    token: &str,
    entry_id: Uuid,
    file_name: Option<&str>,
    bytes: &[u8],
) -> Response {
    let mut request = client
        .post(app.entry_url(entry_id, "/attachments"))
        .bearer_auth(token)
        .header(CONTENT_TYPE, "application/pdf")
        .body(bytes.to_vec());
    if let Some(file_name) = file_name {
        request = request.header("X-File-Name", file_name);
    }

    request.send().await.unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn draft_then_send_creates_a_reply() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let created = post_entry(
        &app,
        &client,
        &app.admin.token,
        ticket.id.0,
        "draft",
        "We are looking into it",
    )
    .await;

    assert_eq!(created.status(), StatusCode::CREATED);
    let draft = json_body(created).await;
    let draft = &draft["data"];
    assert_eq!(draft["kind"], "draft");
    assert_eq!(draft["ticket_id"], ticket.id.0.to_string());
    assert_eq!(draft["author_member_id"], app.admin.member_id.to_string());
    assert_eq!(draft["body"], "We are looking into it");
    assert_eq!(draft["attachments"], json!([]));
    assert!(draft["created_at"].is_string());
    assert!(draft["updated_at"].is_string());
    let entry_id: Uuid = draft["id"].as_str().unwrap().parse().unwrap();

    let sent = client
        .post(app.entry_url(entry_id, "/send"))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();

    assert_eq!(sent.status(), StatusCode::OK);
    let reply = json_body(sent).await;
    assert_eq!(reply["data"]["id"], entry_id.to_string());
    assert_eq!(reply["data"]["kind"], "reply");
    assert_eq!(reply["data"]["body"], "We are looking into it");
    assert_eq!(
        reply["data"]["author_member_id"],
        app.admin.member_id.to_string()
    );

    let thread = list_entries(&app, &client, &app.viewer.token, ticket.id.0).await;
    assert_eq!(kinds(&thread), ["reply"]);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn note_is_listed_for_members() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    app.seed_entry(EntryKind::ClientMessage, &ticket, "It smokes")
        .await;

    let note = post_entry(
        &app,
        &client,
        &app.admin.token,
        ticket.id.0,
        "note",
        "Called the vendor",
    )
    .await;
    assert_eq!(note.status(), StatusCode::CREATED);
    let note = json_body(note).await;
    assert_eq!(note["data"]["kind"], "internal_note");
    assert_eq!(note["data"]["body"], "Called the vendor");
    assert_eq!(
        note["data"]["author_member_id"],
        app.admin.member_id.to_string()
    );

    let reply = post_entry(
        &app,
        &client,
        &app.admin.token,
        ticket.id.0,
        "reply",
        "Fixed",
    )
    .await;
    assert_eq!(reply.status(), StatusCode::CREATED);
    assert_eq!(json_body(reply).await["data"]["kind"], "reply");

    let thread = list_entries(&app, &client, &app.viewer.token, ticket.id.0).await;
    assert_eq!(kinds(&thread), ["client_message", "internal_note", "reply"]);
    assert_eq!(thread["data"][0]["author_member_id"], Value::Null);
    assert_eq!(thread["data"][1]["body"], "Called the vendor");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn unknown_kind_is_422() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let response = post_entry(
        &app,
        &client,
        &app.admin.token,
        ticket.id.0,
        "whisper",
        "psst",
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let thread = list_entries(&app, &client, &app.admin.token, ticket.id.0).await;
    assert_eq!(thread["data"], json!([]));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn editing_a_sent_reply_is_409() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let reply = app
        .seed_entry(EntryKind::Reply, &ticket, "Already sent")
        .await;

    let response = client
        .patch(app.entry_url(reply.id.0, ""))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "body": "Rewritten" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let thread = list_entries(&app, &client, &app.admin.token, ticket.id.0).await;
    assert_eq!(thread["data"][0]["body"], "Already sent");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn deleting_a_reply_is_409() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let reply = app
        .seed_entry(EntryKind::Reply, &ticket, "Already sent")
        .await;

    let response = client
        .delete(app.entry_url(reply.id.0, ""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let thread = list_entries(&app, &client, &app.admin.token, ticket.id.0).await;
    assert_eq!(kinds(&thread), ["reply"]);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn a_draft_can_be_edited_then_deleted() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "First take")
        .await;

    let edited = client
        .patch(app.entry_url(draft.id.0, ""))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "body": "Second take" }))
        .send()
        .await
        .unwrap();

    assert_eq!(edited.status(), StatusCode::OK);
    let edited = json_body(edited).await;
    assert_eq!(edited["data"]["id"], draft.id.0.to_string());
    assert_eq!(edited["data"]["kind"], "draft");
    assert_eq!(edited["data"]["body"], "Second take");

    let deleted = client
        .delete(app.entry_url(draft.id.0, ""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
    let thread = list_entries(&app, &client, &app.admin.token, ticket.id.0).await;
    assert_eq!(thread["data"], json!([]));

    let again = client
        .delete(app.entry_url(draft.id.0, ""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(again.status(), StatusCode::NOT_FOUND);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn member_without_view_support_cannot_list_entries() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let anonymous = client
        .get(app.entries_url(ticket.id.0))
        .send()
        .await
        .unwrap();
    let outsider = client
        .get(app.entries_url(ticket.id.0))
        .bearer_auth(&app.outsider.token)
        .send()
        .await
        .unwrap();

    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(outsider.status(), StatusCode::FORBIDDEN);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn view_without_manage_cannot_add_a_note() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let response = post_entry(
        &app,
        &client,
        &app.viewer.token,
        ticket.id.0,
        "note",
        "Sneaky",
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let thread = list_entries(&app, &client, &app.admin.token, ticket.id.0).await;
    assert_eq!(thread["data"], json!([]));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn entry_of_another_organization_is_404() {
    let app = harness::start().await;
    let client = Client::new();
    let foreign_ticket = app
        .seed_ticket(app.other_organization_id, "Foreign", json!({}))
        .await;
    let foreign_entry = app
        .seed_entry(EntryKind::ClientMessage, &foreign_ticket, "Hello")
        .await;

    let list = client
        .get(app.entries_url(foreign_ticket.id.0))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    let add = post_entry(
        &app,
        &client,
        &app.admin.token,
        foreign_ticket.id.0,
        "note",
        "Intruder",
    )
    .await;
    let edit = client
        .patch(app.entry_url(foreign_entry.id.0, ""))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "body": "Intruder" }))
        .send()
        .await
        .unwrap();
    let delete = client
        .delete(app.entry_url(foreign_entry.id.0, ""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    let send = client
        .post(app.entry_url(foreign_entry.id.0, "/send"))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    let attach = upload(
        &app,
        &client,
        &app.admin.token,
        foreign_entry.id.0,
        Some("intruder.pdf"),
        b"%PDF-1.4",
    )
    .await;

    for response in [list, add, edit, delete, send, attach] {
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    let own_ticket = app.seed_ticket(app.organization_id, "Own", json!({})).await;
    let own = list_entries(&app, &client, &app.admin.token, own_ticket.id.0).await;
    assert_eq!(own["data"], json!([]));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn attaching_to_a_client_message_is_409() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let message = app
        .seed_entry(EntryKind::ClientMessage, &ticket, "It smokes")
        .await;

    let response = upload(
        &app,
        &client,
        &app.admin.token,
        message.id.0,
        Some("smoke.pdf"),
        b"%PDF-1.4",
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let thread = list_entries(&app, &client, &app.admin.token, ticket.id.0).await;
    assert_eq!(thread["data"][0]["attachments"], json!([]));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn a_percent_encoded_file_name_is_decoded() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Devis", json!({}))
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "Voir pièce jointe")
        .await;

    let response = upload(
        &app,
        &client,
        &app.admin.token,
        draft.id.0,
        Some("devis-%C3%A9t%C3%A9.pdf"),
        b"%PDF-1.4 devis",
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let created: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(created["data"]["file_name"], "devis-été.pdf");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn attachment_upload_is_recorded_on_the_entry_and_never_exposes_the_key() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "See attached")
        .await;

    let response = upload(
        &app,
        &client,
        &app.admin.token,
        draft.id.0,
        Some("quote.pdf"),
        b"%PDF-1.4 quote",
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = response.text().await.unwrap();
    let created: Value = serde_json::from_str(&created).unwrap();
    let attachment = &created["data"];
    assert_eq!(attachment["file_name"], "quote.pdf");
    assert_eq!(attachment["mime_type"], "application/pdf");
    assert_eq!(attachment["size_bytes"], 14);
    assert!(attachment["id"].is_string());
    assert!(attachment["created_at"].is_string());

    let attachment_id: Uuid = attachment["id"].as_str().unwrap().parse().unwrap();
    let file_key: String =
        sqlx::query_scalar("SELECT file_key FROM support_ticket_attachments WHERE id = $1")
            .bind(attachment_id)
            .fetch_one(&app.pool)
            .await
            .unwrap();

    let listed = client
        .get(app.entries_url(ticket.id.0))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let thread: Value = serde_json::from_str(&listed).unwrap();
    let attachments = thread["data"][0]["attachments"].as_array().unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0], *attachment);

    let sent = client
        .post(app.entry_url(draft.id.0, "/send"))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let sent_json: Value = serde_json::from_str(&sent).unwrap();
    assert_eq!(sent_json["data"]["attachments"][0]["id"], attachment["id"]);

    for text in [created.to_string(), listed, sent] {
        assert!(!text.contains("file_key"));
        assert!(!text.contains(&file_key));
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn attachment_url_returns_a_presigned_url() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "See attached")
        .await;
    let created = upload(
        &app,
        &client,
        &app.admin.token,
        draft.id.0,
        Some("quote.pdf"),
        b"%PDF-1.4 quote",
    )
    .await;
    let created = json_body(created).await;
    let attachment_id: Uuid = created["data"]["id"].as_str().unwrap().parse().unwrap();

    let response = client
        .get(app.attachment_url(attachment_id))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let url = body["data"]["url"].as_str().unwrap();
    assert!(url.starts_with("http"));
    assert!(body["data"]["expires_at"].is_string());
    assert_eq!(body["data"].as_object().unwrap().len(), 2);

    let object = client.get(url).send().await.unwrap();
    assert_eq!(object.status(), StatusCode::OK);
    assert_eq!(object.bytes().await.unwrap().as_ref(), b"%PDF-1.4 quote");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn member_without_view_support_cannot_read_an_attachment_url() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let message = app
        .seed_entry(EntryKind::ClientMessage, &ticket, "Photo")
        .await;
    let attachment = app.seed_client_attachment(&message).await;

    let anonymous = client
        .get(app.attachment_url(attachment.id.0))
        .send()
        .await
        .unwrap();
    let outsider = client
        .get(app.attachment_url(attachment.id.0))
        .bearer_auth(&app.outsider.token)
        .send()
        .await
        .unwrap();

    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(outsider.status(), StatusCode::FORBIDDEN);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn attachment_url_of_another_organization_is_404() {
    let app = harness::start().await;
    let client = Client::new();
    let foreign_ticket = app
        .seed_ticket(app.other_organization_id, "Foreign", json!({}))
        .await;
    let foreign_message = app
        .seed_entry(EntryKind::ClientMessage, &foreign_ticket, "Photo")
        .await;
    let foreign_attachment = app.seed_client_attachment(&foreign_message).await;

    let response = client
        .get(app.attachment_url(foreign_attachment.id.0))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    let unknown = client
        .get(app.attachment_url(Uuid::now_v7()))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);

    let own_ticket = app.seed_ticket(app.organization_id, "Own", json!({})).await;
    let own_message = app
        .seed_entry(EntryKind::ClientMessage, &own_ticket, "Photo")
        .await;
    let own_attachment = app.seed_client_attachment(&own_message).await;
    let own = client
        .get(app.attachment_url(own_attachment.id.0))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(own.status(), StatusCode::OK);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn view_without_manage_cannot_attach_a_file() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "See attached")
        .await;

    let response = upload(
        &app,
        &client,
        &app.viewer.token,
        draft.id.0,
        Some("quote.pdf"),
        b"%PDF-1.4 quote",
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let thread = list_entries(&app, &client, &app.admin.token, ticket.id.0).await;
    assert_eq!(thread["data"][0]["attachments"], json!([]));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn an_upload_without_a_file_name_is_refused() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "See attached")
        .await;

    let missing = upload(
        &app,
        &client,
        &app.admin.token,
        draft.id.0,
        None,
        b"%PDF-1.4 quote",
    )
    .await;
    let blank = upload(
        &app,
        &client,
        &app.admin.token,
        draft.id.0,
        Some("   "),
        b"%PDF-1.4 quote",
    )
    .await;

    assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
    assert_eq!(blank.status(), StatusCode::BAD_REQUEST);
    let thread = list_entries(&app, &client, &app.admin.token, ticket.id.0).await;
    assert_eq!(thread["data"][0]["attachments"], json!([]));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn an_empty_upload_is_refused() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "See attached")
        .await;

    let response = upload(
        &app,
        &client,
        &app.admin.token,
        draft.id.0,
        Some("quote.pdf"),
        b"",
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let thread = list_entries(&app, &client, &app.admin.token, ticket.id.0).await;
    assert_eq!(thread["data"][0]["attachments"], json!([]));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn an_upload_larger_than_the_default_body_limit_is_accepted() {
    let app = harness::start().await;
    let client = Client::new();
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "See attached")
        .await;
    let bytes = vec![7_u8; 3 * 1024 * 1024];

    let response = upload(
        &app,
        &client,
        &app.admin.token,
        draft.id.0,
        Some("scan.pdf"),
        &bytes,
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = json_body(response).await;
    assert_eq!(created["data"]["size_bytes"], 3 * 1024 * 1024);
    app.cleanup().await;
}
