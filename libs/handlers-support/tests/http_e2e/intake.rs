use authz::Subject;
use mestier_core::{AssignTicketCommand, EntryKind, MemberId, Ticket, TicketEntry, TicketStatus};
use reqwest::{Client, Method, Response, StatusCode, header::CONTENT_TYPE};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App};

const UNAUTHORIZED_BODY: &str =
    r#"{"code":"E_UNAUTHORIZED","message":"Unauthorized","status":401}"#;
const NOT_FOUND_BODY: &str =
    r#"{"code":"E_NOT_FOUND","message":"Resource not found","status":404}"#;

const FORBIDDEN_KEYS: [&str; 7] = [
    "author_member_id",
    "author",
    "assignee",
    "assigned_at",
    "file_key",
    "storage_key",
    "updated_at",
];

pub(crate) struct Intake<'a> {
    app: &'a App,
    client: Client,
    key: String,
}

async fn issue_key(app: &App, client: &Client, token: &str, organization_id: Uuid) -> Value {
    let response = client
        .post(app.api_keys_url_in(organization_id, ""))
        .bearer_auth(token)
        .header("x-forwarded-for", format!("issue-key-{}", Uuid::now_v7()))
        .json(&json!({ "name": "Autharie" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = response.json().await.unwrap();

    body["data"].clone()
}

impl<'a> Intake<'a> {
    pub(crate) async fn new(app: &'a App) -> Self {
        let client = Client::new();
        let created = issue_key(
            app,
            &client,
            &app.organization_admin.token,
            app.organization_id,
        )
        .await;

        Self {
            app,
            client,
            key: created["key"].as_str().unwrap().to_owned(),
        }
    }

    async fn for_other_organization(app: &'a App) -> Self {
        let client = Client::new();
        let created = issue_key(
            app,
            &client,
            &app.other_organization_admin.token,
            app.other_organization_id,
        )
        .await;

        Self {
            app,
            client,
            key: created["key"].as_str().unwrap().to_owned(),
        }
    }

    pub(crate) async fn create(&self, body: &Value) -> Response {
        self.client
            .post(self.app.intake_url("/tickets"))
            .bearer_auth(&self.key)
            .json(body)
            .send()
            .await
            .unwrap()
    }

    pub(crate) async fn list(&self, query: &str) -> Response {
        self.client
            .get(self.app.intake_url(&format!("/tickets{query}")))
            .bearer_auth(&self.key)
            .send()
            .await
            .unwrap()
    }

    pub(crate) async fn view(&self, ticket_id: Uuid) -> Response {
        self.client
            .get(self.app.intake_url(&format!("/tickets/{ticket_id}")))
            .bearer_auth(&self.key)
            .send()
            .await
            .unwrap()
    }

    pub(crate) async fn message(&self, ticket_id: Uuid, body: &Value) -> Response {
        self.client
            .post(
                self.app
                    .intake_url(&format!("/tickets/{ticket_id}/messages")),
            )
            .bearer_auth(&self.key)
            .json(body)
            .send()
            .await
            .unwrap()
    }

    async fn upload(
        &self,
        ticket_id: Uuid,
        query: &str,
        file_name: Option<&str>,
        bytes: &[u8],
    ) -> Response {
        self.upload_as(ticket_id, query, file_name, Some("application/pdf"), bytes)
            .await
    }

    async fn upload_as(
        &self,
        ticket_id: Uuid,
        query: &str,
        file_name: Option<&str>,
        content_type: Option<&str>,
        bytes: &[u8],
    ) -> Response {
        let mut request = self
            .client
            .post(
                self.app
                    .intake_url(&format!("/tickets/{ticket_id}/attachments{query}")),
            )
            .bearer_auth(&self.key)
            .body(bytes.to_vec());
        if let Some(content_type) = content_type {
            request = request.header(CONTENT_TYPE, content_type);
        }
        if let Some(file_name) = file_name {
            request = request.header("X-File-Name", file_name);
        }

        request.send().await.unwrap()
    }

    pub(crate) async fn attach(
        &self,
        ticket_id: Uuid,
        entry_id: Uuid,
        file_name: &str,
    ) -> Response {
        self.upload(
            ticket_id,
            &format!("?entry_id={entry_id}"),
            Some(file_name),
            b"%PDF-1.4 intake",
        )
        .await
    }

    async fn url(&self, ticket_id: Uuid, attachment_id: Uuid) -> Response {
        self.client
            .get(self.app.intake_url(&format!(
                "/tickets/{ticket_id}/attachments/{attachment_id}/url"
            )))
            .bearer_auth(&self.key)
            .send()
            .await
            .unwrap()
    }
}

pub(crate) async fn read(response: Response) -> (StatusCode, String) {
    let status = response.status();

    (status, response.text().await.unwrap())
}

pub(crate) fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn sorted_keys(value: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();

    keys
}

pub(crate) fn id_of(value: &Value) -> Uuid {
    value["id"].as_str().unwrap().parse().unwrap()
}

fn ticket_body(subject: &str) -> Value {
    json!({
        "subject": subject,
        "body": format!("{subject} body"),
        "custom_fields": {},
    })
}

fn assert_no_forbidden_key(text: &str) {
    for key in FORBIDDEN_KEYS {
        assert!(!text.contains(key), "`{key}` found in {text}");
    }
}

async fn ticket_rows(app: &App, organization_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM support_tickets WHERE org_id = $1")
        .bind(organization_id)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

async fn attachment_rows(app: &App, organization_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM support_ticket_attachments WHERE org_id = $1")
        .bind(organization_id)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

async fn entry_rows(app: &App, organization_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM support_ticket_entries WHERE org_id = $1")
        .bind(organization_id)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

async fn file_keys(app: &App, organization_id: Uuid) -> Vec<String> {
    sqlx::query_scalar("SELECT file_key FROM support_ticket_attachments WHERE org_id = $1")
        .bind(organization_id)
        .fetch_all(&app.pool)
        .await
        .unwrap()
}

async fn attachment_of(app: &App, entry: &TicketEntry) -> Uuid {
    sqlx::query_scalar("SELECT id FROM support_ticket_attachments WHERE entry_id = $1")
        .bind(entry.id.0)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

struct Thread {
    ticket: Ticket,
    client_message: TicketEntry,
    note: TicketEntry,
    reply: TicketEntry,
    draft: TicketEntry,
}

async fn seed_thread(app: &App, organization_id: Uuid, subject: &str) -> Thread {
    let ticket = app
        .seed_ticket(organization_id, subject, json!({ "team": "tech" }))
        .await;
    let client_message = app
        .seed_entry(EntryKind::ClientMessage, &ticket, "CLIENT-MESSAGE-ONE")
        .await;
    let note = app
        .seed_entry(EntryKind::InternalNote, &ticket, "SECRET-NOTE-TEXT")
        .await;
    let reply = app
        .seed_entry(EntryKind::Reply, &ticket, "REPLY-TEXT-TWO")
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "SECRET-DRAFT-TEXT")
        .await;
    app.seed_client_attachment(&client_message).await;
    app.seed_member_attachment(&note, "secret-note.png").await;
    app.seed_member_attachment(&reply, "reply-doc.png").await;
    app.seed_member_attachment(&draft, "secret-draft.png").await;

    Thread {
        ticket,
        client_message,
        note,
        reply,
        draft,
    }
}

async fn assign_to_admin(app: &App, ticket: &Ticket) {
    app.state
        .usecase
        .assign_support_ticket(AssignTicketCommand {
            actor: Subject::system(),
            organization_id: ticket.organization_id,
            ticket_id: ticket.id,
            assignee: Some(MemberId(app.admin.member_id)),
        })
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn creating_a_ticket_answers_201_then_200_with_the_same_ticket_for_the_same_external_id() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let other = Intake::for_other_organization(&app).await;
    let body = json!({
        "subject": "  Le déploiement échoue  ",
        "body": "Le déploiement dpl_8f2a91 échoue.",
        "custom_fields": { "plan": "pro", "seats": 12 },
        "external_id": "autharie-ticket-1",
    });

    let (first_status, first_text) = read(intake.create(&body).await).await;
    let first = parse(&first_text);
    let again = json!({
        "subject": "Autre sujet",
        "body": "Autre corps",
        "custom_fields": {},
        "external_id": "autharie-ticket-1",
    });
    let (second_status, second_text) = read(intake.create(&again).await).await;
    let second = parse(&second_text);
    let (other_status, other_text) = read(other.create(&body).await).await;
    let foreign = parse(&other_text);

    assert_eq!(first_status, StatusCode::CREATED);
    assert_eq!(
        sorted_keys(&first["data"]),
        [
            "badges",
            "body",
            "created_at",
            "custom_fields",
            "external_id",
            "id",
            "number",
            "status",
            "subject"
        ]
    );
    assert_eq!(first["data"]["subject"], "Le déploiement échoue");
    assert_eq!(first["data"]["body"], "Le déploiement dpl_8f2a91 échoue.");
    assert_eq!(first["data"]["status"], "new");
    assert_eq!(first["data"]["number"], 1);
    assert_eq!(first["data"]["badges"], json!([]));
    assert_eq!(first["data"]["external_id"], "autharie-ticket-1");
    assert_eq!(
        first["data"]["custom_fields"],
        json!({ "plan": "pro", "seats": 12 })
    );
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(second["data"]["id"], first["data"]["id"]);
    assert_eq!(second["data"]["number"], first["data"]["number"]);
    assert_eq!(second["data"]["subject"], "Le déploiement échoue");
    assert_eq!(sorted_keys(&second["data"]), sorted_keys(&first["data"]));
    assert_eq!(ticket_rows(&app, app.organization_id).await, 1);
    assert_eq!(other_status, StatusCode::CREATED);
    assert_ne!(foreign["data"]["id"], first["data"]["id"]);
    assert_eq!(ticket_rows(&app, app.other_organization_id).await, 1);
    for text in [&first_text, &second_text, &other_text] {
        assert_no_forbidden_key(text);
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_ticket_without_external_id_is_created_every_time() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;

    let first = intake.create(&ticket_body("Premier")).await;
    let second = intake.create(&ticket_body("Premier")).await;
    let (_, text) = read(first).await;
    let first = parse(&text);
    let (_, text) = read(second).await;
    let second = parse(&text);

    assert_ne!(first["data"]["id"], second["data"]["id"]);
    assert_eq!(first["data"]["external_id"], Value::Null);
    assert_eq!(second["data"]["number"], 2);
    assert_eq!(ticket_rows(&app, app.organization_id).await, 2);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_organization_comes_from_the_key_and_never_from_the_request() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let mut body = ticket_body("Pirate");
    body["organization_id"] = json!(app.other_organization_id);
    body["org_id"] = json!(app.other_organization_id);

    let (status, text) = read(intake.create(&body).await).await;

    assert_eq!(status, StatusCode::CREATED);
    let created = parse(&text);
    assert!(!text.contains(&app.organization_id.to_string()));
    assert!(!text.contains("organization_id"));
    assert_eq!(ticket_rows(&app, app.organization_id).await, 1);
    assert_eq!(ticket_rows(&app, app.other_organization_id).await, 0);
    let stored: Uuid = sqlx::query_scalar("SELECT org_id FROM support_tickets WHERE id = $1")
        .bind(id_of(&created["data"]))
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(stored, app.organization_id);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn custom_fields_must_be_an_object_within_the_limits_or_the_answer_is_422() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let too_many_keys: serde_json::Map<String, Value> = (0..65)
        .map(|index| (format!("k{index}"), json!(index)))
        .collect();
    let long_key = "k".repeat(65);
    let cases = [
        ("array", json!(["team"])),
        ("string", json!("team")),
        ("null", Value::Null),
        ("number", json!(3)),
        ("boolean", json!(true)),
        ("65 keys", Value::Object(too_many_keys)),
        ("empty key", json!({ "": 1 })),
        ("65 characters key", json!({ long_key: 1 })),
        ("more than 16 KiB", json!({ "blob": "x".repeat(16 * 1024) })),
    ];

    for (label, custom_fields) in cases {
        let response = intake
            .create(&json!({ "subject": "Sujet", "body": "Corps", "custom_fields": custom_fields }))
            .await;

        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "{label}"
        );
    }
    let missing = intake
        .create(&json!({ "subject": "Sujet", "body": "Corps" }))
        .await;
    assert_eq!(
        missing.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "missing"
    );
    assert_eq!(ticket_rows(&app, app.organization_id).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_blank_or_missing_subject_or_body_is_422() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let cases = [
        (
            "blank subject",
            json!({ "subject": "   ", "body": "Corps", "custom_fields": {} }),
        ),
        (
            "empty subject",
            json!({ "subject": "", "body": "Corps", "custom_fields": {} }),
        ),
        (
            "blank body",
            json!({ "subject": "Sujet", "body": "  \n\t ", "custom_fields": {} }),
        ),
        (
            "empty body",
            json!({ "subject": "Sujet", "body": "", "custom_fields": {} }),
        ),
        (
            "missing subject",
            json!({ "body": "Corps", "custom_fields": {} }),
        ),
        (
            "missing body",
            json!({ "subject": "Sujet", "custom_fields": {} }),
        ),
        (
            "subject not text",
            json!({ "subject": 3, "body": "Corps", "custom_fields": {} }),
        ),
        (
            "external id not text",
            json!({ "subject": "Sujet", "body": "Corps", "custom_fields": {}, "external_id": 4 }),
        ),
    ];

    for (label, body) in cases {
        let response = intake.create(&body).await;

        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "{label}"
        );
    }
    assert_eq!(ticket_rows(&app, app.organization_id).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn text_the_database_cannot_hold_is_refused_with_422_and_never_a_500() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Existant", json!({}))
        .await;
    let nul = "a\u{0}b";

    let responses = [
        (
            "subject",
            intake
                .create(&json!({ "subject": nul, "body": "Corps", "custom_fields": {} }))
                .await,
        ),
        (
            "body",
            intake
                .create(&json!({ "subject": "Sujet", "body": nul, "custom_fields": {} }))
                .await,
        ),
        (
            "external id",
            intake
                .create(&json!({ "subject": "Sujet", "body": "Corps", "custom_fields": {}, "external_id": nul }))
                .await,
        ),
        (
            "custom field value",
            intake
                .create(&json!({ "subject": "Sujet", "body": "Corps", "custom_fields": { "key": nul } }))
                .await,
        ),
        (
            "nested custom field value",
            intake
                .create(&json!({ "subject": "Sujet", "body": "Corps", "custom_fields": { "key": [{ "deep": nul }] } }))
                .await,
        ),
        (
            "custom field key",
            intake
                .create(&json!({ "subject": "Sujet", "body": "Corps", "custom_fields": { nul: 1 } }))
                .await,
        ),
        (
            "message",
            intake.message(ticket.id.0, &json!({ "body": nul })).await,
        ),
        (
            "filter key",
            intake.list("?custom_field_key=a%00b&custom_field_value=x").await,
        ),
        (
            "filter value",
            intake.list("?custom_field_key=plan&custom_field_value=a%00b").await,
        ),
    ];

    for (label, response) in responses {
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "{label}"
        );
    }
    assert_eq!(ticket_rows(&app, app.organization_id).await, 1);
    assert_eq!(entry_rows(&app, app.organization_id).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_external_id_over_255_characters_is_422_before_anything_is_written() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;

    for length in [256, 3000] {
        let (status, text) = read(
            intake
                .create(&json!({
                    "subject": "Sujet",
                    "body": "Corps",
                    "custom_fields": {},
                    "external_id": "x".repeat(length),
                }))
                .await,
        )
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{length}");
        assert!(!text.contains(&"x".repeat(20)), "{text}");
    }
    assert_eq!(ticket_rows(&app, app.organization_id).await, 0);

    let longest = "x".repeat(255);
    let (accepted_status, accepted_text) = read(
        intake
            .create(&json!({
                "subject": "Sujet",
                "body": "Corps",
                "custom_fields": {},
                "external_id": longest,
            }))
            .await,
    )
    .await;
    assert_eq!(accepted_status, StatusCode::CREATED);
    assert_eq!(parse(&accepted_text)["data"]["external_id"], longest);
    assert_eq!(ticket_rows(&app, app.organization_id).await, 1);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_ticket_view_and_the_listing_carry_neither_a_note_nor_a_draft_nor_a_member() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let thread = seed_thread(&app, app.organization_id, "Printer on fire").await;
    assign_to_admin(&app, &thread.ticket).await;
    let keys = file_keys(&app, app.organization_id).await;
    assert_eq!(keys.len(), 4);

    let (view_status, view_text) = read(intake.view(thread.ticket.id.0).await).await;
    let (list_status, list_text) = read(intake.list("").await).await;

    assert_eq!(view_status, StatusCode::OK);
    assert_eq!(list_status, StatusCode::OK);
    let view = parse(&view_text);
    let ticket = &view["data"];
    assert_eq!(
        sorted_keys(ticket),
        [
            "badges",
            "body",
            "created_at",
            "custom_fields",
            "entries",
            "external_id",
            "id",
            "number",
            "status",
            "subject"
        ]
    );
    assert_eq!(ticket["id"], thread.ticket.id.0.to_string());
    assert_eq!(ticket["custom_fields"], json!({ "team": "tech" }));
    let entries = ticket["entries"].as_array().unwrap();
    let kinds: Vec<&str> = entries
        .iter()
        .map(|entry| entry["kind"].as_str().unwrap())
        .collect();
    let bodies: Vec<&str> = entries
        .iter()
        .map(|entry| entry["body"].as_str().unwrap())
        .collect();
    let ids: Vec<String> = entries
        .iter()
        .map(|entry| entry["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(kinds, ["client_message", "reply"]);
    assert_eq!(bodies, ["CLIENT-MESSAGE-ONE", "REPLY-TEXT-TWO"]);
    assert_eq!(
        ids,
        [
            thread.client_message.id.0.to_string(),
            thread.reply.id.0.to_string()
        ]
    );
    for entry in entries {
        assert_eq!(
            sorted_keys(entry),
            ["attachments", "body", "created_at", "id", "kind"]
        );
        let attachments = entry["attachments"].as_array().unwrap();
        assert_eq!(attachments.len(), 1);
        assert_eq!(
            sorted_keys(&attachments[0]),
            ["created_at", "file_name", "id", "mime_type", "size_bytes"]
        );
    }
    assert_eq!(entries[0]["attachments"][0]["file_name"], "photo.png");
    assert_eq!(entries[1]["attachments"][0]["file_name"], "reply-doc.png");

    let listing = parse(&list_text);
    let items = listing["data"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], ticket["id"]);
    assert!(items[0].get("entries").is_none());
    assert_eq!(
        sorted_keys(&items[0]),
        [
            "badges",
            "body",
            "created_at",
            "custom_fields",
            "external_id",
            "id",
            "number",
            "status",
            "subject"
        ]
    );
    assert!(listing["pagination"].is_object());

    let forbidden_values = [
        "SECRET-NOTE-TEXT",
        "SECRET-DRAFT-TEXT",
        "secret-note.png",
        "secret-draft.png",
        "uploads/",
    ];
    let member_id = app.admin.member_id.to_string();
    let hidden_entries = [thread.note.id.0.to_string(), thread.draft.id.0.to_string()];
    for text in [&view_text, &list_text] {
        assert_no_forbidden_key(text);
        for value in forbidden_values {
            assert!(!text.contains(value), "`{value}` found in {text}");
        }
        for value in keys.iter().chain(&hidden_entries) {
            assert!(!text.contains(value.as_str()), "`{value}` found in {text}");
        }
        assert!(!text.contains(&member_id), "member id found in {text}");
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_ticket_of_another_organization_or_an_unknown_one_is_404_with_the_same_body() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let foreign = app
        .seed_ticket(app.other_organization_id, "Foreign", json!({}))
        .await;

    let (foreign_status, foreign_text) = read(intake.view(foreign.id.0).await).await;
    let (unknown_status, unknown_text) = read(intake.view(Uuid::now_v7()).await).await;

    assert_eq!(foreign_status, StatusCode::NOT_FOUND);
    assert_eq!(unknown_status, StatusCode::NOT_FOUND);
    assert_eq!(foreign_text, unknown_text);
    assert_eq!(foreign_text, NOT_FOUND_BODY);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_key_of_organization_b_reaches_nothing_of_organization_a() {
    let app = harness::start().await;
    let intake_a = Intake::new(&app).await;
    let intake_b = Intake::for_other_organization(&app).await;
    let thread = seed_thread(&app, app.organization_id, "Secret of A").await;
    let own = app
        .seed_ticket(app.other_organization_id, "Own of B", json!({}))
        .await;
    let own_message = app
        .seed_entry(EntryKind::ClientMessage, &own, "Message of B")
        .await;
    let own_attachment = app.seed_client_attachment(&own_message).await;
    let a_attachment = attachment_of(&app, &thread.client_message).await;
    let entries_before = entry_rows(&app, app.organization_id).await;
    let attachments_before = attachment_rows(&app, app.organization_id).await;

    let view = intake_b.view(thread.ticket.id.0).await;
    let message = intake_b
        .message(thread.ticket.id.0, &json!({ "body": "Intrus" }))
        .await;
    let upload = intake_b
        .attach(thread.ticket.id.0, thread.client_message.id.0, "intrus.pdf")
        .await;
    let url = intake_b.url(thread.ticket.id.0, a_attachment).await;
    let own_ticket_foreign_entry = intake_b
        .attach(own.id.0, thread.client_message.id.0, "intrus.pdf")
        .await;
    let own_ticket_foreign_attachment = intake_b.url(own.id.0, a_attachment).await;
    let reverse = intake_a.view(own.id.0).await;
    let reverse_url = intake_a.url(thread.ticket.id.0, own_attachment.id.0).await;

    for (label, response) in [
        ("view", view),
        ("message", message),
        ("upload", upload),
        ("url", url),
        ("own ticket, foreign entry", own_ticket_foreign_entry),
        (
            "own ticket, foreign attachment",
            own_ticket_foreign_attachment,
        ),
        ("reverse view", reverse),
        ("reverse url", reverse_url),
    ] {
        let (status, text) = read(response).await;

        assert_eq!(status, StatusCode::NOT_FOUND, "{label}");
        assert_eq!(text, NOT_FOUND_BODY, "{label}");
    }

    let (_, listing) = read(intake_b.list("").await).await;
    let listing = parse(&listing);
    let subjects: Vec<&str> = listing["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["subject"].as_str().unwrap())
        .collect();
    assert_eq!(subjects, ["Own of B"]);
    assert_eq!(entry_rows(&app, app.organization_id).await, entries_before);
    assert_eq!(
        attachment_rows(&app, app.organization_id).await,
        attachments_before
    );
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_custom_field_filter_compares_the_top_level_value_as_text() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let intake_b = Intake::for_other_organization(&app).await;
    app.seed_ticket(app.organization_id, "Number", json!({ "plan": 42 }))
        .await;
    app.seed_ticket(app.organization_id, "Text", json!({ "plan": "42" }))
        .await;
    app.seed_ticket(app.organization_id, "Other value", json!({ "plan": "7" }))
        .await;
    app.seed_ticket(app.organization_id, "Other key", json!({ "tier": "42" }))
        .await;
    app.seed_ticket(
        app.organization_id,
        "Nested",
        json!({ "wrapper": { "plan": "42" } }),
    )
    .await;
    app.seed_ticket(
        app.other_organization_id,
        "Foreign",
        json!({ "plan": "42" }),
    )
    .await;

    let (status, text) = read(
        intake
            .list("?custom_field_key=plan&custom_field_value=42")
            .await,
    )
    .await;
    let (_, foreign_text) = read(
        intake_b
            .list("?custom_field_key=plan&custom_field_value=42")
            .await,
    )
    .await;
    let (_, everything) = read(intake.list("").await).await;
    let (_, ignored_filters) = read(intake.list("?status=resolved&badge=none").await).await;

    assert_eq!(status, StatusCode::OK);
    let subjects = |text: &str| -> Vec<String> {
        parse(text)["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["subject"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(subjects(&text), ["Text", "Number"]);
    assert_eq!(parse(&text)["pagination"]["total"], 2);
    assert_eq!(subjects(&foreign_text), ["Foreign"]);
    assert_eq!(
        subjects(&everything),
        ["Nested", "Other key", "Other value", "Text", "Number"]
    );
    assert_eq!(subjects(&ignored_filters), subjects(&everything));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn only_one_of_the_two_custom_field_parameters_is_422() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    app.seed_ticket(app.organization_id, "Sujet", json!({ "plan": "42" }))
        .await;

    let key_only = intake.list("?custom_field_key=plan").await;
    let value_only = intake.list("?custom_field_value=42").await;
    let both = intake
        .list("?custom_field_key=plan&custom_field_value=42")
        .await;

    assert_eq!(key_only.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(value_only.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(both.status(), StatusCode::OK);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_listing_is_newest_first_and_paginated() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    for subject in ["Premier", "Deuxième", "Troisième"] {
        app.seed_ticket(app.organization_id, subject, json!({}))
            .await;
    }

    let (_, first_page) = read(intake.list("?per_page=2").await).await;
    let (_, second_page) = read(intake.list("?per_page=2&page=2").await).await;
    let (status, far) = read(intake.list("?page=18446744073709551615").await).await;

    let first_page = parse(&first_page);
    let second_page = parse(&second_page);
    let subjects = |page: &Value| -> Vec<String> {
        page["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["subject"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(subjects(&first_page), ["Troisième", "Deuxième"]);
    assert_eq!(subjects(&second_page), ["Premier"]);
    assert_eq!(first_page["pagination"]["per_page"], 2);
    assert_eq!(first_page["pagination"]["current_page"], 1);
    assert_eq!(first_page["pagination"]["total"], 3);
    assert_eq!(first_page["pagination"]["last_page"], 2);
    assert_eq!(second_page["pagination"]["current_page"], 2);
    assert_eq!(status, StatusCode::OK);
    assert_eq!(parse(&far)["data"], json!([]));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_client_message_is_created_listed_and_never_carries_an_author() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let (status, text) = read(
        intake
            .message(ticket.id.0, &json!({ "body": "Still smoking" }))
            .await,
    )
    .await;
    let (_, view) = read(intake.view(ticket.id.0).await).await;

    assert_eq!(status, StatusCode::CREATED);
    let created = parse(&text);
    assert_eq!(
        sorted_keys(&created["data"]),
        ["attachments", "body", "created_at", "id", "kind"]
    );
    assert_eq!(created["data"]["kind"], "client_message");
    assert_eq!(created["data"]["body"], "Still smoking");
    assert_eq!(created["data"]["attachments"], json!([]));
    let view = parse(&view);
    let entries = view["data"]["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["id"], created["data"]["id"]);
    assert_eq!(view["data"]["status"], "new");
    assert_no_forbidden_key(&text);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_blank_or_missing_message_is_422_and_nothing_is_written() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    for body in [
        json!({ "body": "" }),
        json!({ "body": "   \n " }),
        json!({}),
        json!({ "body": 4 }),
        json!({ "body": null }),
    ] {
        let response = intake.message(ticket.id.0, &body).await;

        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "{body}"
        );
    }
    let unknown = intake
        .message(Uuid::now_v7(), &json!({ "body": "Bonjour" }))
        .await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    assert_eq!(entry_rows(&app, app.organization_id).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_message_reopens_a_resolved_or_waiting_ticket_through_http() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let resolved = app
        .seed_ticket(app.organization_id, "Resolved", json!({}))
        .await;
    let waiting = app
        .seed_ticket(app.organization_id, "Waiting", json!({}))
        .await;
    let in_progress = app
        .seed_ticket(app.organization_id, "In progress", json!({}))
        .await;
    app.set_status(&resolved, TicketStatus::Resolved).await;
    app.set_status(&waiting, TicketStatus::WaitingCustomer)
        .await;
    app.set_status(&in_progress, TicketStatus::InProgress).await;

    for ticket in [&resolved, &waiting, &in_progress] {
        let response = intake
            .message(ticket.id.0, &json!({ "body": "Toujours en panne" }))
            .await;
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    for ticket in [&resolved, &waiting, &in_progress] {
        let (_, text) = read(intake.view(ticket.id.0).await).await;

        assert_eq!(
            parse(&text)["data"]["status"],
            "in_progress",
            "{}",
            ticket.subject
        );
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn a_file_is_uploaded_to_a_client_message_and_read_back_through_the_url_route() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Devis", json!({}))
        .await;
    let (_, message) = read(
        intake
            .message(ticket.id.0, &json!({ "body": "Voir le devis" }))
            .await,
    )
    .await;
    let message_id = id_of(&parse(&message)["data"]);

    let (status, text) = read(
        intake
            .upload(
                ticket.id.0,
                &format!("?entry_id={message_id}"),
                Some("devis-%C3%A9t%C3%A9.pdf"),
                b"%PDF-1.4 devis",
            )
            .await,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let created = parse(&text);
    assert_eq!(
        sorted_keys(&created["data"]),
        ["created_at", "file_name", "id", "mime_type", "size_bytes"]
    );
    assert_eq!(created["data"]["file_name"], "devis-été.pdf");
    assert_eq!(created["data"]["mime_type"], "application/pdf");
    assert_eq!(created["data"]["size_bytes"], 14);
    let keys = file_keys(&app, app.organization_id).await;
    assert_eq!(keys.len(), 1);
    assert!(!text.contains(&keys[0]));
    assert_no_forbidden_key(&text);

    let (_, view) = read(intake.view(ticket.id.0).await).await;
    assert!(!view.contains(&keys[0]));
    let view = parse(&view);
    let attachments = &view["data"]["entries"][0]["attachments"];
    assert_eq!(attachments.as_array().unwrap().len(), 1);
    assert_eq!(attachments[0], created["data"]);

    let attachment_id = id_of(&created["data"]);
    let (url_status, url_text) = read(intake.url(ticket.id.0, attachment_id).await).await;
    assert_eq!(url_status, StatusCode::OK);
    assert!(!url_text.contains("file_key"));
    let url = parse(&url_text);
    assert_eq!(sorted_keys(&url["data"]), ["expires_at", "url"]);
    assert!(url["data"]["expires_at"].is_string());
    let object = Client::new()
        .get(url["data"]["url"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(object.status(), StatusCode::OK);
    assert_eq!(object.bytes().await.unwrap().as_ref(), b"%PDF-1.4 devis");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn a_malformed_upload_is_refused_and_attaches_nothing() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Devis", json!({}))
        .await;
    let message = app
        .seed_entry(EntryKind::ClientMessage, &ticket, "Voir le devis")
        .await;
    let valid = format!("?entry_id={}", message.id.0);

    let missing_entry = intake
        .upload(ticket.id.0, "", Some("a.pdf"), b"%PDF-1.4")
        .await;
    let malformed_entry = intake
        .upload(
            ticket.id.0,
            "?entry_id=not-a-uuid",
            Some("a.pdf"),
            b"%PDF-1.4",
        )
        .await;
    let blank_entry = intake
        .upload(ticket.id.0, "?entry_id=", Some("a.pdf"), b"%PDF-1.4")
        .await;
    let twice = intake
        .upload(
            ticket.id.0,
            &format!("?entry_id={}&entry_id={}", message.id.0, message.id.0),
            Some("a.pdf"),
            b"%PDF-1.4",
        )
        .await;
    let missing_name = intake.upload(ticket.id.0, &valid, None, b"%PDF-1.4").await;
    let blank_name = intake
        .upload(ticket.id.0, &valid, Some("   "), b"%PDF-1.4")
        .await;
    let empty_body = intake.upload(ticket.id.0, &valid, Some("a.pdf"), b"").await;

    assert_eq!(missing_entry.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(malformed_entry.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(blank_entry.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(twice.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(missing_name.status(), StatusCode::BAD_REQUEST);
    assert_eq!(blank_name.status(), StatusCode::BAD_REQUEST);
    assert_eq!(empty_body.status(), StatusCode::BAD_REQUEST);
    assert_eq!(attachment_rows(&app, app.organization_id).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn the_stored_mime_type_never_follows_a_client_type_that_can_run_script() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Devis", json!({}))
        .await;
    let cases = [
        (Some("text/html"), "application/octet-stream"),
        (Some("TEXT/HTML; charset=utf-8"), "application/octet-stream"),
        (Some("image/svg+xml"), "application/octet-stream"),
        (Some("application/xhtml+xml"), "application/octet-stream"),
        (Some("garbage"), "application/octet-stream"),
        (None, "application/octet-stream"),
        (Some("image/png"), "image/png"),
        (Some("IMAGE/PNG; name=a.png"), "image/png"),
        (Some("text/plain; charset=utf-8"), "text/plain"),
    ];

    let mut expected = Vec::new();
    for (index, (content_type, stored)) in cases.iter().enumerate() {
        let message = app
            .seed_entry(
                EntryKind::ClientMessage,
                &ticket,
                &format!("message {index}"),
            )
            .await;
        let (status, text) = read(
            intake
                .upload_as(
                    ticket.id.0,
                    &format!("?entry_id={}", message.id.0),
                    Some("file.bin"),
                    *content_type,
                    b"<script>alert(1)</script>",
                )
                .await,
        )
        .await;

        assert_eq!(status, StatusCode::CREATED, "{content_type:?}");
        assert_eq!(
            parse(&text)["data"]["mime_type"],
            *stored,
            "{content_type:?}"
        );
        expected.push((message.id.0.to_string(), *stored));
    }

    let (_, view) = read(intake.view(ticket.id.0).await).await;
    let view = parse(&view);
    for (entry_id, stored) in expected {
        let entry = view["data"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == entry_id.as_str())
            .unwrap();
        assert_eq!(entry["attachments"][0]["mime_type"], stored);
    }
    let stored_types: Vec<String> =
        sqlx::query_scalar("SELECT mime_type FROM support_ticket_attachments WHERE org_id = $1")
            .bind(app.organization_id)
            .fetch_all(&app.pool)
            .await
            .unwrap();
    assert_eq!(stored_types.len(), cases.len());
    for stored in &stored_types {
        assert!(
            !stored.contains("html") && !stored.contains("svg") && !stored.contains("xml"),
            "{stored}"
        );
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn an_upload_with_a_name_that_cannot_be_displayed_safely_is_422_and_stores_nothing() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Devis", json!({}))
        .await;
    let message = app
        .seed_entry(EntryKind::ClientMessage, &ticket, "Voir le devis")
        .await;
    let valid = format!("?entry_id={}", message.id.0);
    let too_long = "a".repeat(256);
    let names = [
        "evil%0d%0ainjected.png",
        "line%0abreak.png",
        "tab%09name.png",
        "del%7fname.png",
        "bidi%e2%80%aefdp.exe",
        "isolate%e2%81%a6name.png",
        too_long.as_str(),
    ];

    for name in names {
        let (status, text) = read(intake.attach(ticket.id.0, message.id.0, name).await).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{name}");
        assert!(!text.contains("evil"), "{text}");
        assert!(!text.contains("injected"), "{text}");
        assert!(!text.contains(&too_long), "{text}");
    }

    assert_eq!(attachment_rows(&app, app.organization_id).await, 0);
    assert!(file_keys(&app, app.organization_id).await.is_empty());

    let longest_accepted = "a".repeat(255);
    let accepted = intake
        .upload(
            ticket.id.0,
            &valid,
            Some(&longest_accepted),
            b"%PDF-1.4 intake",
        )
        .await;
    assert_eq!(accepted.status(), StatusCode::CREATED);
    assert_eq!(attachment_rows(&app, app.organization_id).await, 1);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn an_upload_over_the_maximum_size_is_refused_and_attaches_nothing() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Devis", json!({}))
        .await;
    let message = app
        .seed_entry(EntryKind::ClientMessage, &ticket, "Voir le devis")
        .await;
    let too_large = vec![7_u8; app.state.args.file_storage.max_upload_bytes as usize + 1];

    let oversize = intake
        .upload(
            ticket.id.0,
            &format!("?entry_id={}", message.id.0),
            Some("scan.pdf"),
            &too_large,
        )
        .await;
    let unauthenticated = Client::new()
        .post(app.intake_url(&format!("/tickets/{}/attachments", ticket.id.0)))
        .body(too_large)
        .send()
        .await
        .unwrap();

    assert!(
        matches!(
            oversize.status(),
            StatusCode::PAYLOAD_TOO_LARGE | StatusCode::UNPROCESSABLE_ENTITY
        ),
        "{}",
        oversize.status()
    );
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(attachment_rows(&app, app.organization_id).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn only_a_client_message_of_that_ticket_takes_a_file() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let thread = seed_thread(&app, app.organization_id, "Printer on fire").await;
    let other_ticket = app
        .seed_ticket(app.organization_id, "Another ticket", json!({}))
        .await;
    let other_message = app
        .seed_entry(EntryKind::ClientMessage, &other_ticket, "Elsewhere")
        .await;
    let foreign_ticket = app
        .seed_ticket(app.other_organization_id, "Foreign", json!({}))
        .await;
    let foreign_message = app
        .seed_entry(EntryKind::ClientMessage, &foreign_ticket, "Abroad")
        .await;
    let before = attachment_rows(&app, app.organization_id).await;

    let mut texts = Vec::new();
    for (label, entry_id) in [
        ("reply", thread.reply.id.0),
        ("internal note", thread.note.id.0),
        ("draft", thread.draft.id.0),
        ("client message of another ticket", other_message.id.0),
        (
            "client message of another organization",
            foreign_message.id.0,
        ),
        ("unknown entry", Uuid::now_v7()),
    ] {
        let (status, text) = read(intake.attach(thread.ticket.id.0, entry_id, "a.pdf").await).await;

        assert_eq!(status, StatusCode::NOT_FOUND, "{label}");
        texts.push(text);
    }
    let unknown_ticket = intake
        .attach(Uuid::now_v7(), thread.client_message.id.0, "a.pdf")
        .await;
    let (status, text) = read(unknown_ticket).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    texts.push(text);

    assert!(texts.iter().all(|text| text == NOT_FOUND_BODY), "{texts:?}");
    assert_eq!(attachment_rows(&app, app.organization_id).await, before);
    assert_eq!(attachment_rows(&app, app.other_organization_id).await, 0);

    let accepted = intake
        .attach(thread.ticket.id.0, thread.client_message.id.0, "a.pdf")
        .await;
    assert_eq!(accepted.status(), StatusCode::CREATED);
    assert_eq!(attachment_rows(&app, app.organization_id).await, before + 1);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn the_url_route_serves_client_visible_attachments_only_and_hides_the_rest_with_one_404() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let intake_b = Intake::for_other_organization(&app).await;
    let thread = seed_thread(&app, app.organization_id, "Printer on fire").await;
    let other = seed_thread(&app, app.organization_id, "Another ticket").await;
    let foreign_ticket = app
        .seed_ticket(app.other_organization_id, "Foreign", json!({}))
        .await;
    let foreign_message = app
        .seed_entry(EntryKind::ClientMessage, &foreign_ticket, "Abroad")
        .await;
    app.seed_client_attachment(&foreign_message).await;
    let client_attachment = attachment_of(&app, &thread.client_message).await;
    let reply_attachment = attachment_of(&app, &thread.reply).await;
    let note_attachment = attachment_of(&app, &thread.note).await;
    let draft_attachment = attachment_of(&app, &thread.draft).await;
    let other_ticket_attachment = attachment_of(&app, &other.client_message).await;
    let foreign_attachment = attachment_of(&app, &foreign_message).await;

    let client = intake.url(thread.ticket.id.0, client_attachment).await;
    let reply = intake.url(thread.ticket.id.0, reply_attachment).await;

    assert_eq!(client.status(), StatusCode::OK);
    assert_eq!(reply.status(), StatusCode::OK);
    let (_, client_text) = read(client).await;
    let (_, reply_text) = read(reply).await;
    for text in [&client_text, &reply_text] {
        let body = parse(text);
        assert_eq!(sorted_keys(&body["data"]), ["expires_at", "url"]);
        assert!(body["data"]["url"].as_str().unwrap().starts_with("http"));
        assert_no_forbidden_key(text);
    }

    let mut refusals = Vec::new();
    for (label, response) in [
        (
            "internal note",
            intake.url(thread.ticket.id.0, note_attachment).await,
        ),
        (
            "draft",
            intake.url(thread.ticket.id.0, draft_attachment).await,
        ),
        (
            "attachment of another ticket",
            intake
                .url(thread.ticket.id.0, other_ticket_attachment)
                .await,
        ),
        (
            "attachment of another organization",
            intake.url(thread.ticket.id.0, foreign_attachment).await,
        ),
        (
            "ticket of another organization",
            intake.url(foreign_ticket.id.0, foreign_attachment).await,
        ),
        (
            "unknown attachment",
            intake.url(thread.ticket.id.0, Uuid::now_v7()).await,
        ),
        (
            "unknown ticket",
            intake.url(Uuid::now_v7(), client_attachment).await,
        ),
        (
            "other key, own ticket, foreign attachment",
            intake_b.url(foreign_ticket.id.0, client_attachment).await,
        ),
        (
            "other key, foreign ticket",
            intake_b.url(thread.ticket.id.0, client_attachment).await,
        ),
    ] {
        let (status, text) = read(response).await;

        assert_eq!(status, StatusCode::NOT_FOUND, "{label}");
        refusals.push((label, text));
    }
    for (label, text) in &refusals {
        assert_eq!(text, NOT_FOUND_BODY, "{label}");
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn an_attachment_of_a_draft_becomes_readable_once_the_draft_is_sent() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;
    let draft = app
        .seed_entry(EntryKind::Draft, &ticket, "Draft text")
        .await;
    let attachment = app.seed_member_attachment(&draft, "later.png").await;

    let before = intake.url(ticket.id.0, attachment.id.0).await;
    app.send_draft(&draft).await;
    let after = intake.url(ticket.id.0, attachment.id.0).await;

    assert_eq!(before.status(), StatusCode::NOT_FOUND);
    assert_eq!(after.status(), StatusCode::OK);
    app.cleanup().await;
}

fn six_routes(
    ticket_id: Uuid,
    attachment_id: Uuid,
) -> Vec<(&'static str, Method, String, &'static str)> {
    vec![
        (
            "createIntakeTicket",
            Method::POST,
            "/tickets".to_owned(),
            "{",
        ),
        (
            "listIntakeTickets",
            Method::GET,
            "/tickets?custom_field_key=only".to_owned(),
            "",
        ),
        (
            "getIntakeTicket",
            Method::GET,
            format!("/tickets/{ticket_id}"),
            "",
        ),
        (
            "postIntakeMessage",
            Method::POST,
            format!("/tickets/{ticket_id}/messages"),
            "{",
        ),
        (
            "attachIntakeFile",
            Method::POST,
            format!("/tickets/{ticket_id}/attachments"),
            "",
        ),
        (
            "getIntakeAttachmentUrl",
            Method::GET,
            format!("/tickets/{ticket_id}/attachments/{attachment_id}/url"),
            "",
        ),
    ]
}

async fn attempt(
    app: &App,
    client: &Client,
    method: &Method,
    path: &str,
    body: &'static str,
    authorization: &[String],
) -> (StatusCode, String) {
    let mut request = client
        .request(method.clone(), app.intake_url(path))
        .header(CONTENT_TYPE, "application/json")
        .body(body);
    for value in authorization {
        request = request.header("authorization", value);
    }

    read(request.send().await.unwrap()).await
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn every_refusal_of_the_key_is_the_same_401_on_all_six_routes_before_anything_else() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let client = Client::new();
    let revoked = issue_key(
        &app,
        &client,
        &app.organization_admin.token,
        app.organization_id,
    )
    .await;
    let revoked_id = id_of(&revoked);
    let revocation = client
        .delete(app.api_keys_url(&format!("/{revoked_id}")))
        .bearer_auth(&app.organization_admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(revocation.status(), StatusCode::NO_CONTENT);
    let thread = seed_thread(&app, app.organization_id, "Printer on fire").await;
    let (prefix, secret) = intake.key.split_once('.').unwrap();
    let flipped = if secret.starts_with('0') { '1' } else { '0' };
    let unknown_prefix = if prefix.starts_with('0') { '1' } else { '0' };
    let tickets_before = ticket_rows(&app, app.organization_id).await;
    let entries_before = entry_rows(&app, app.organization_id).await;
    let attachments_before = attachment_rows(&app, app.organization_id).await;

    let attempts: Vec<(&str, Vec<String>)> = vec![
        ("no header", vec![]),
        ("wrong scheme", vec![format!("Basic {}", intake.key)]),
        ("no scheme", vec![intake.key.clone()]),
        ("double space", vec![format!("Bearer  {}", intake.key)]),
        ("empty key", vec!["Bearer ".to_owned()]),
        ("malformed key", vec!["Bearer not-a-key".to_owned()]),
        (
            "wrong secret",
            vec![format!("Bearer {prefix}.{flipped}{}", &secret[1..])],
        ),
        (
            "unknown prefix",
            vec![format!("Bearer {unknown_prefix}{}.{secret}", &prefix[1..])],
        ),
        (
            "revoked key",
            vec![format!("Bearer {}", revoked["key"].as_str().unwrap())],
        ),
        (
            "ferriskey user token",
            vec![format!("Bearer {}", app.admin.token)],
        ),
        (
            "ferriskey organization administrator token",
            vec![format!("Bearer {}", app.organization_admin.token)],
        ),
        (
            "two authorization headers",
            vec![
                format!("Bearer {}", intake.key),
                format!("Bearer {}", intake.key),
            ],
        ),
    ];

    for (route, method, path, body) in six_routes(thread.ticket.id.0, Uuid::now_v7()) {
        for (label, authorization) in &attempts {
            let (status, text) = attempt(&app, &client, &method, &path, body, authorization).await;

            assert_eq!(status, StatusCode::UNAUTHORIZED, "{route}: {label}");
            assert_eq!(text, UNAUTHORIZED_BODY, "{route}: {label}");
        }
    }

    assert_eq!(ticket_rows(&app, app.organization_id).await, tickets_before);
    assert_eq!(entry_rows(&app, app.organization_id).await, entries_before);
    assert_eq!(
        attachment_rows(&app, app.organization_id).await,
        attachments_before
    );
    let last_use: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT last_used_at FROM support_api_keys WHERE id = $1")
            .bind(revoked_id)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(last_use, None);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_key_of_a_soft_deleted_organization_is_the_same_401_on_all_six_routes() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let thread = seed_thread(&app, app.organization_id, "Printer on fire").await;
    assert_eq!(
        intake.view(thread.ticket.id.0).await.status(),
        StatusCode::OK
    );
    sqlx::query("UPDATE organizations SET deleted_at = now() WHERE id = $1")
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .unwrap();
    let authorization = vec![format!("Bearer {}", intake.key)];

    for (route, method, path, body) in six_routes(thread.ticket.id.0, Uuid::now_v7()) {
        let (status, text) =
            attempt(&app, &intake.client, &method, &path, body, &authorization).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED, "{route}");
        assert_eq!(text, UNAUTHORIZED_BODY, "{route}");
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_intake_key_never_opens_a_member_route_while_it_opens_the_intake_ones() {
    let app = harness::start().await;
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    let member_routes = [
        app.tickets_url(""),
        app.ticket_url(ticket.id.0, ""),
        app.entries_url(ticket.id.0),
        app.api_keys_url(""),
    ];
    for url in member_routes {
        let response = intake
            .client
            .get(&url)
            .bearer_auth(&intake.key)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{url}");
    }
    assert_eq!(intake.view(ticket.id.0).await.status(), StatusCode::OK);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_intake_routes_are_behind_the_ip_rate_limiter() {
    let app = harness::start_with_rate_limit(2).await;
    let client = Client::new();
    let intake = Intake::new(&app).await;
    let ticket = app
        .seed_ticket(app.organization_id, "Printer on fire", json!({}))
        .await;

    for (route, method, path, body) in six_routes(ticket.id.0, Uuid::now_v7()) {
        let address = format!("rate-limit-{}", Uuid::now_v7());
        let mut statuses = Vec::new();
        let mut last = None;
        for _ in 0..3 {
            let response = client
                .request(method.clone(), app.intake_url(&path))
                .header("x-forwarded-for", &address)
                .header(CONTENT_TYPE, "application/json")
                .bearer_auth(&intake.key)
                .body(body)
                .send()
                .await
                .unwrap();
            statuses.push(response.status());
            last = Some(response);
        }

        let blocked = last.unwrap();
        assert_ne!(statuses[0], StatusCode::TOO_MANY_REQUESTS, "{route}");
        assert_ne!(statuses[1], StatusCode::TOO_MANY_REQUESTS, "{route}");
        assert_eq!(statuses[2], StatusCode::TOO_MANY_REQUESTS, "{route}");
        assert_eq!(blocked.headers()["x-ratelimit-limit"], "2", "{route}");
        assert!(blocked.headers().contains_key("retry-after"), "{route}");
    }

    let address = format!("rate-limit-{}", Uuid::now_v7());
    let mut anonymous = Vec::new();
    for _ in 0..3 {
        let response = client
            .get(app.intake_url("/tickets"))
            .header("x-forwarded-for", &address)
            .send()
            .await
            .unwrap();
        anonymous.push(response.status());
    }
    assert_eq!(
        anonymous,
        [
            StatusCode::UNAUTHORIZED,
            StatusCode::UNAUTHORIZED,
            StatusCode::TOO_MANY_REQUESTS
        ]
    );
    app.cleanup().await;
}
