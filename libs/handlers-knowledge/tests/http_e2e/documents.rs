use reqwest::Method;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App};

const EMPTY: fn() -> Value = || json!({ "version": 1, "blocks": [] });

fn paragraph(id: &str, text: &str) -> Value {
    json!({
        "id": id, "type": "paragraph", "props": {},
        "content": [{ "type": "text", "text": text, "styles": {} }],
        "children": []
    })
}

fn blocks(blocks: Vec<Value>) -> Value {
    json!({ "version": 1, "blocks": blocks })
}

async fn create_document(app: &App, title: &str) -> String {
    let created = app
        .knowledge(
            Method::POST,
            &app.manager.token,
            app.organization_id,
            "/articles/documents",
            Some(json!({ "title": title })),
        )
        .await;
    assert_eq!(created.status, 201, "{}", created.body);
    created.body["data"]["id"].as_str().unwrap().to_owned()
}

async fn lock(app: &App, token: &str, id: &str) -> crate::harness::Reply {
    app.knowledge(
        Method::POST,
        token,
        app.organization_id,
        &format!("/articles/{id}/lock"),
        None,
    )
    .await
}

async fn save(
    app: &App,
    token: &str,
    id: &str,
    content: Value,
    revision: i64,
) -> crate::harness::Reply {
    app.knowledge(
        Method::PUT,
        token,
        app.organization_id,
        &format!("/articles/{id}/content"),
        Some(json!({ "content": content, "revision": revision })),
    )
    .await
}

async fn read(app: &App, id: &str) -> Value {
    let read = app
        .knowledge(
            Method::GET,
            &app.manager.token,
            app.organization_id,
            &format!("/articles/{id}"),
            None,
        )
        .await;
    assert_eq!(read.status, 200, "{}", read.body);
    read.body["data"].clone()
}

async fn patch(app: &App, token: &str, id: &str, payload: Value) -> crate::harness::Reply {
    app.knowledge(
        Method::PATCH,
        token,
        app.organization_id,
        &format!("/articles/{id}"),
        Some(payload),
    )
    .await
}

fn block_types(article: &Value) -> Vec<String> {
    article["content"]["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|block| block["type"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_document_created_in_a_folder_is_listed_there_empty_at_revision_zero() {
    let app = harness::start().await;
    let folder = app
        .knowledge(
            Method::POST,
            &app.manager.token,
            app.organization_id,
            "/folders",
            Some(json!({ "name": "Chantiers" })),
        )
        .await;
    assert_eq!(folder.status, 201, "{}", folder.body);
    let folder_id = folder.body["data"]["id"].as_str().unwrap().to_owned();

    let created = app
        .knowledge(
            Method::POST,
            &app.manager.token,
            app.organization_id,
            "/articles/documents",
            Some(json!({ "title": "Compte rendu", "folder_id": folder_id, "tags": ["Réunion"] })),
        )
        .await;
    assert_eq!(created.status, 201, "{}", created.body);
    let document = &created.body["data"];
    assert_eq!(document["kind"], "document");
    assert_eq!(document["revision"], 0);
    assert_eq!(document["content"], EMPTY());
    assert_eq!(document["lock"], Value::Null);
    assert_eq!(document["folder_id"], folder_id);
    assert_eq!(document["tags"], json!(["réunion"]));

    let listed = app
        .knowledge(
            Method::GET,
            &app.manager.token,
            app.organization_id,
            &format!("/articles?folder={folder_id}"),
            None,
        )
        .await;
    assert_eq!(listed.status, 200, "{}", listed.body);
    assert_eq!(listed.body["data"][0]["id"], document["id"]);
    assert_eq!(listed.body["data"][0]["kind"], "document");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_save_under_the_lock_bumps_the_revision_writes_the_markdown_and_is_searchable() {
    let app = harness::start().await;
    let id = create_document(&app, "Plan de pose").await;
    let granted = lock(&app, &app.manager.token, &id).await;
    assert_eq!(granted.status, 200, "{}", granted.body);
    assert_eq!(
        granted.body["data"]["holder_id"],
        app.manager.user_id.to_string()
    );
    assert_eq!(granted.body["data"]["revision"], 0);

    let content = blocks(vec![
        json!({ "id": "h", "type": "heading", "props": { "level": 1 },
            "content": [{ "type": "text", "text": "Plan", "styles": {} }], "children": [] }),
        paragraph("p", "Livraison de rosiers mardi"),
    ]);
    let saved = save(&app, &app.manager.token, &id, content.clone(), 0).await;
    assert_eq!(saved.status, 200, "{}", saved.body);
    assert_eq!(saved.body["data"]["revision"], 1);

    let document = read(&app, &id).await;
    assert_eq!(document["content"], content);
    assert_eq!(document["revision"], 1);
    assert_eq!(document["body"], "# Plan\n\nLivraison de rosiers mardi");
    assert_eq!(
        document["lock"]["holder_id"],
        app.manager.user_id.to_string()
    );

    let found = app
        .knowledge(
            Method::GET,
            &app.manager.token,
            app.organization_id,
            "/articles?q=rosier",
            None,
        )
        .await;
    assert_eq!(found.status, 200, "{}", found.body);
    assert_eq!(found.body["data"][0]["id"], id);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn two_people_taking_the_lock_at_once_leave_exactly_one_holder() {
    let app = harness::start().await;
    for round in 0..5 {
        let id = create_document(&app, &format!("Course {round}")).await;
        let (first, second) = tokio::join!(
            lock(&app, &app.manager.token, &id),
            lock(&app, &app.colleague.token, &id)
        );
        let mut statuses = [first.status, second.status];
        statuses.sort_unstable();
        assert_eq!(
            statuses,
            [200, 409],
            "round {round}: {} / {}",
            first.body,
            second.body
        );
        let (winner, loser, winner_id) = if first.status == 200 {
            (&first, &second, app.manager.user_id)
        } else {
            (&second, &first, app.colleague.user_id)
        };
        assert_eq!(winner.body["data"]["holder_id"], winner_id.to_string());
        assert_eq!(loser.body["holder_id"], winner_id.to_string());
        assert!(loser.body["expires_at"].is_string());
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_lock_lasts_ninety_seconds_and_a_lapsed_one_is_taken_without_error() {
    let app = harness::start().await;
    let id = create_document(&app, "Délai").await;
    let granted = lock(&app, &app.manager.token, &id).await;
    let expires =
        chrono::DateTime::parse_from_rfc3339(granted.body["data"]["expires_at"].as_str().unwrap())
            .unwrap();
    let left = (expires.to_utc() - chrono::Utc::now()).num_seconds();
    assert!((80..=90).contains(&left), "{left}");

    let refused = lock(&app, &app.colleague.token, &id).await;
    assert_eq!(refused.status, 409);

    sqlx::query("UPDATE knowledge_articles SET lock_expires_at = now() - interval '91 seconds' WHERE id = $1")
        .bind(Uuid::parse_str(&id).unwrap())
        .execute(&app.pool)
        .await
        .unwrap();
    let taken = lock(&app, &app.colleague.token, &id).await;
    assert_eq!(taken.status, 200, "{}", taken.body);
    assert_eq!(
        taken.body["data"]["holder_id"],
        app.colleague.user_id.to_string()
    );

    let released = app
        .knowledge(
            Method::DELETE,
            &app.manager.token,
            app.organization_id,
            &format!("/articles/{id}/lock"),
            None,
        )
        .await;
    assert_eq!(released.status, 204);
    assert_eq!(
        read(&app, &id).await["lock"]["holder_id"],
        app.colleague.user_id.to_string()
    );
    let freed = app
        .knowledge(
            Method::DELETE,
            &app.colleague.token,
            app.organization_id,
            &format!("/articles/{id}/lock"),
            None,
        )
        .await;
    assert_eq!(freed.status, 204);
    assert_eq!(read(&app, &id).await["lock"], Value::Null);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_save_without_the_lock_with_anothers_or_with_a_stale_revision_changes_nothing() {
    let app = harness::start().await;
    let id = create_document(&app, "Refus").await;
    let content = blocks(vec![paragraph("p", "texte")]);

    let without = save(&app, &app.manager.token, &id, content.clone(), 0).await;
    assert_eq!(without.status, 409, "{}", without.body);
    assert_eq!(without.body["reason"], "lock_lost");

    assert_eq!(lock(&app, &app.colleague.token, &id).await.status, 200);
    let other = save(&app, &app.manager.token, &id, content.clone(), 0).await;
    assert_eq!(other.status, 409);
    assert_eq!(other.body["reason"], "lock_lost");

    let stale = save(&app, &app.colleague.token, &id, content.clone(), 4).await;
    assert_eq!(stale.status, 409);
    assert_eq!(stale.body["reason"], "revision_conflict");

    let document = read(&app, &id).await;
    assert_eq!(document["revision"], 0);
    assert_eq!(document["content"], EMPTY());
    assert_eq!(document["body"], "");

    let ok = save(&app, &app.colleague.token, &id, content, 0).await;
    assert_eq!(ok.status, 200, "{}", ok.body);
    let again = save(&app, &app.colleague.token, &id, blocks(vec![]), 0).await;
    assert_eq!(again.status, 409);
    assert_eq!(again.body["reason"], "revision_conflict");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn invalid_content_is_refused_with_422_and_nothing_changes() {
    let app = harness::start().await;
    let id = create_document(&app, "Invalide").await;
    assert_eq!(lock(&app, &app.manager.token, &id).await.status, 200);

    let nested = |levels: usize| {
        let mut block = paragraph(&format!("n{levels}"), "x");
        for level in (1..levels).rev() {
            let mut parent = paragraph(&format!("n{level}"), "x");
            parent["children"] = json!([block]);
            block = parent;
        }
        blocks(vec![block])
    };
    let invalid = vec![
        (
            "unknown type",
            blocks(vec![
                json!({ "id": "a", "type": "video", "props": {}, "content": [], "children": [] }),
            ]),
        ),
        (
            "heading level 4",
            blocks(vec![
                json!({ "id": "a", "type": "heading", "props": { "level": 4 },
                "content": [], "children": [] }),
            ]),
        ),
        (
            "javascript link",
            blocks(vec![json!({ "id": "a", "type": "paragraph", "props": {},
                "content": [{ "type": "link", "href": "javascript:alert(1)", "content": [] }],
                "children": [] })]),
        ),
        (
            "duplicate id",
            blocks(vec![paragraph("a", "x"), paragraph("a", "y")]),
        ),
        ("seven levels", nested(7)),
        (
            "5001 blocks",
            blocks(
                (0..5001)
                    .map(|n| paragraph(&format!("b{n}"), "x"))
                    .collect(),
            ),
        ),
        (
            "styled code",
            blocks(vec![json!({ "id": "a", "type": "code", "props": {},
                "content": [{ "type": "text", "text": "x", "styles": { "bold": true } }],
                "children": [] })]),
        ),
    ];
    for (name, content) in invalid {
        let refused = save(&app, &app.manager.token, &id, content, 0).await;
        assert_eq!(refused.status, 422, "{name}: {}", refused.body);
    }
    assert_eq!(
        save(&app, &app.manager.token, &id, nested(6), 0)
            .await
            .status,
        200
    );

    let too_big = blocks(vec![paragraph("a", &"x".repeat(1_200_000))]);
    let refused = save(&app, &app.manager.token, &id, too_big, 1).await;
    assert_eq!(refused.status, 413);
    assert_eq!(read(&app, &id).await["revision"], 1);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_element_of_another_organization_is_not_found_and_nothing_is_saved() {
    let app = harness::start().await;
    let intruder_org = app.intruder_organization_id;
    let foreign_customer = app.seed_customer(intruder_org, "Étranger").await;
    let foreign_file = app.seed_file(intruder_org, "Secret").await;
    let foreign_table = app
        .call(
            Method::POST,
            &app.intruder.token,
            intruder_org,
            "",
            Some(json!({ "name": "Table étrangère" })),
        )
        .await;
    assert_eq!(foreign_table.status, 201, "{}", foreign_table.body);
    let table_id = foreign_table.body["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let view_id = foreign_table.body["data"]["views"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let own_customer = app.seed_customer(app.organization_id, "Dupont").await;

    let id = create_document(&app, "Frontière").await;
    assert_eq!(lock(&app, &app.manager.token, &id).await.status, 200);
    let foreign = vec![
        json!({ "id": "a", "type": "file", "props": { "article_id": foreign_file },
            "content": [], "children": [] }),
        json!({ "id": "a", "type": "image", "props": { "source": "drive", "id": foreign_file },
            "content": [], "children": [] }),
        json!({ "id": "a", "type": "image", "props": { "source": "photo", "id": Uuid::now_v7() },
            "content": [], "children": [] }),
        json!({ "id": "a", "type": "table_view", "props": { "table_id": table_id, "view_id": view_id },
            "content": [], "children": [] }),
        json!({ "id": "a", "type": "paragraph", "props": {},
            "content": [{ "type": "mention", "resource_type": "customer",
                "resource_id": foreign_customer, "label": "Étranger" }], "children": [] }),
    ];
    for block in foreign {
        let refused = save(
            &app,
            &app.manager.token,
            &id,
            blocks(vec![block.clone()]),
            0,
        )
        .await;
        assert_eq!(refused.status, 404, "{block}: {}", refused.body);
    }
    let document = read(&app, &id).await;
    assert_eq!(document["revision"], 0);
    assert_eq!(document["content"], EMPTY());

    let own = blocks(vec![json!({ "id": "a", "type": "paragraph", "props": {},
        "content": [{ "type": "mention", "resource_type": "customer",
            "resource_id": own_customer, "label": "Dupont" }], "children": [] })]);
    assert_eq!(
        save(&app, &app.manager.token, &id, own, 0).await.status,
        200
    );

    let intruder_view = app
        .knowledge(
            Method::GET,
            &app.intruder.token,
            intruder_org,
            &format!("/articles/{id}"),
            None,
        )
        .await;
    assert_eq!(intruder_view.status, 404);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_document_whose_table_was_deleted_still_reads_and_saves() {
    let app = harness::start().await;
    let table = app
        .as_manager(Method::POST, "", Some(json!({ "name": "Suivi chantiers" })))
        .await;
    let table_id = table.body["data"]["id"].as_str().unwrap().to_owned();
    let view_id = table.body["data"]["views"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let id = create_document(&app, "Avec table").await;
    assert_eq!(lock(&app, &app.manager.token, &id).await.status, 200);
    let content = blocks(vec![json!({ "id": "t", "type": "table_view",
        "props": { "table_id": table_id, "view_id": view_id }, "content": [], "children": [] })]);
    assert_eq!(
        save(&app, &app.manager.token, &id, content.clone(), 0)
            .await
            .status,
        200
    );

    let deleted = app
        .as_manager(Method::DELETE, &format!("/{table_id}"), None)
        .await;
    assert_eq!(deleted.status, 204, "{}", deleted.body);

    let document = read(&app, &id).await;
    assert_eq!(document["content"], content);
    let mut longer = content;
    longer["blocks"]
        .as_array_mut()
        .unwrap()
        .push(paragraph("p", "toujours là"));
    let saved = save(&app, &app.manager.token, &id, longer, 1).await;
    assert_eq!(saved.status, 200, "{}", saved.body);
    assert_eq!(saved.body["data"]["revision"], 2);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_markdown_patch_stores_blocks_and_bumps_the_revision_unless_someone_else_edits() {
    let app = harness::start().await;
    let customer = app.seed_customer(app.organization_id, "Dupont").await;
    let id = create_document(&app, "Markdown").await;
    let markdown = format!(
        "# Titre\n\n- un\n- [x] fait\n\n> citation\n\n```rust\nlet a = 1;\n```\n\n\
Voir [le site](https://exemple.fr) et [Dupont](mestier:customer/{customer})."
    );
    let patched = patch(&app, &app.manager.token, &id, json!({ "body": markdown })).await;
    assert_eq!(patched.status, 200, "{}", patched.body);
    let data = &patched.body["data"];
    assert_eq!(data["revision"], 1);
    assert_eq!(
        block_types(data),
        [
            "heading",
            "bullet_item",
            "check_item",
            "quote",
            "code",
            "paragraph"
        ]
    );
    assert_eq!(data["content"]["blocks"][2]["props"]["checked"], true);
    assert_eq!(data["content"]["blocks"][4]["props"]["language"], "rust");
    let inlines = data["content"]["blocks"][5]["content"].as_array().unwrap();
    assert!(
        inlines
            .iter()
            .any(|i| i["type"] == "link" && i["href"] == "https://exemple.fr")
    );
    assert!(
        inlines
            .iter()
            .any(|i| i["type"] == "mention" && i["resource_id"] == customer.to_string())
    );
    assert!(data["body"].as_str().unwrap().contains("# Titre"));

    assert_eq!(lock(&app, &app.colleague.token, &id).await.status, 200);
    let refused = patch(
        &app,
        &app.manager.token,
        &id,
        json!({ "body": "autre texte" }),
    )
    .await;
    assert_eq!(refused.status, 409, "{}", refused.body);
    let title_only = patch(
        &app,
        &app.manager.token,
        &id,
        json!({ "title": "Nouveau titre" }),
    )
    .await;
    assert_eq!(title_only.status, 200, "{}", title_only.body);
    assert_eq!(read(&app, &id).await["revision"], 1);

    let by_holder = patch(
        &app,
        &app.colleague.token,
        &id,
        json!({ "body": "écrit par le détenteur" }),
    )
    .await;
    assert_eq!(by_holder.status, 200, "{}", by_holder.body);
    assert_eq!(by_holder.body["data"]["revision"], 2);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_note_created_from_markdown_is_a_document_and_a_foreign_mention_is_refused() {
    let app = harness::start().await;
    let foreign = app
        .seed_customer(app.intruder_organization_id, "Étranger")
        .await;
    let created = app
        .knowledge(
            Method::POST,
            &app.manager.token,
            app.organization_id,
            "/notes",
            Some(json!({ "title": "Note", "body": "# Idée\n\nUn texte" })),
        )
        .await;
    assert_eq!(created.status, 201, "{}", created.body);
    assert_eq!(created.body["data"]["kind"], "document");
    assert_eq!(block_types(&created.body["data"]), ["heading", "paragraph"]);
    assert_eq!(created.body["data"]["revision"], 0);

    let refused = app
        .knowledge(
            Method::POST,
            &app.manager.token,
            app.organization_id,
            "/notes",
            Some(json!({ "title": "Piège", "body": format!("[X](mestier:customer/{foreign})") })),
        )
        .await;
    assert_eq!(refused.status, 404, "{}", refused.body);

    let blank = app
        .knowledge(
            Method::POST,
            &app.manager.token,
            app.organization_id,
            "/notes",
            Some(json!({ "title": "Vide", "body": "  " })),
        )
        .await;
    assert_eq!(blank.status, 422);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_viewer_reads_the_document_and_its_lock_but_cannot_lock_or_save() {
    let app = harness::start().await;
    let id = create_document(&app, "Lecture").await;
    assert_eq!(lock(&app, &app.manager.token, &id).await.status, 200);

    let read = app
        .knowledge(
            Method::GET,
            &app.viewer.token,
            app.organization_id,
            &format!("/articles/{id}"),
            None,
        )
        .await;
    assert_eq!(read.status, 200, "{}", read.body);
    assert_eq!(
        read.body["data"]["lock"]["holder_id"],
        app.manager.user_id.to_string()
    );
    assert_eq!(read.body["data"]["content"], EMPTY());

    assert_eq!(lock(&app, &app.viewer.token, &id).await.status, 403);
    let saved = save(&app, &app.viewer.token, &id, EMPTY(), 0).await;
    assert_eq!(saved.status, 403);
    let created = app
        .knowledge(
            Method::POST,
            &app.viewer.token,
            app.organization_id,
            "/articles/documents",
            Some(json!({ "title": "Interdit" })),
        )
        .await;
    assert_eq!(created.status, 403);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_file_takes_no_lock_and_no_markdown() {
    let app = harness::start().await;
    let file = app
        .seed_file(app.organization_id, "Tarifs")
        .await
        .to_string();
    assert_eq!(lock(&app, &app.manager.token, &file).await.status, 404);
    let patched = patch(&app, &app.manager.token, &file, json!({ "body": "texte" })).await;
    assert_eq!(patched.status, 422, "{}", patched.body);
    let article = read(&app, &file).await;
    assert_eq!(article["kind"], "file");
    assert!(article.get("content").is_none());
    assert!(article.get("revision").is_none());
    app.cleanup().await;
}
