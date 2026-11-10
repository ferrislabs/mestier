use image::{Rgb, RgbImage, codecs::jpeg::JpegEncoder};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{harness, issuer};

const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

pub(crate) struct Intruder {
    pub(crate) token: String,
    pub(crate) organization_id: Uuid,
    user_id: Uuid,
}

impl Intruder {
    pub(crate) async fn cleanup(&self, pool: &PgPool) {
        for statement in [
            "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
            "DELETE FROM roles WHERE organization_id = $1",
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

pub(crate) async fn seed_intruder(pool: &PgPool) -> Intruder {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-photo-intruder-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("photo-intruder-{user_id}@example.com"))
    .bind(format!("photo-intruder-{user_id}"))
    .bind("Intruder")
    .bind(&sub)
    .execute(pool)
    .await
    .expect("seed the intruder user");

    let organization_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id)
        .bind("Intruder Org")
        .bind(format!("photo-intruder-org-{organization_id}"))
        .bind(user_id)
        .execute(pool)
        .await
        .expect("seed the intruder's organization");
    let member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(organization_id)
    .bind(user_id)
    .bind("Intruder")
    .execute(pool)
    .await
    .expect("seed the intruder's membership");

    let role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, 'owner', $3)",
    )
    .bind(role_id)
    .bind(organization_id)
    .bind(mestier_core::Permissions::ALL.0)
    .execute(pool)
    .await
    .expect("seed the intruder's role");
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(member_id)
        .bind(role_id)
        .execute(pool)
        .await
        .expect("assign the intruder's role");

    Intruder {
        token: issuer::mint(&sub),
        organization_id,
        user_id,
    }
}

fn jpeg(seed: u8, width: u32, height: u32) -> Vec<u8> {
    let image = RgbImage::from_fn(width, height, |x, y| {
        Rgb([
            (x % 251) as u8 ^ seed,
            (y % 241) as u8,
            seed.wrapping_mul(37),
        ])
    });
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, 90)
        .encode_image(&image)
        .expect("encode the test image");
    out
}

fn photos_url(app: &harness::App, organization_id: Uuid, suffix: &str) -> String {
    format!(
        "{}/api/v1/organizations/{organization_id}/photo-library/photos{suffix}",
        app.base_url
    )
}

fn doe_url(app: &harness::App, organization_id: Uuid, doe_id: &str, suffix: &str) -> String {
    format!(
        "{}/api/v1/organizations/{organization_id}/does/{doe_id}{suffix}",
        app.base_url
    )
}

async fn upload(
    app: &harness::App,
    token: &str,
    organization_id: Uuid,
    body: Vec<u8>,
) -> reqwest::Response {
    reqwest::Client::new()
        .post(photos_url(app, organization_id, ""))
        .bearer_auth(token)
        .header("content-type", "image/jpeg")
        .body(body)
        .send()
        .await
        .expect("the api answers the upload call")
}

async fn upload_new(app: &harness::App, seed: u8) -> Value {
    let response = upload(app, &app.token, app.organization_id, jpeg(seed, 64, 48)).await;
    assert_eq!(response.status(), 201);
    response.json().await.expect("the upload answer is json")
}

async fn patch(
    app: &harness::App,
    token: &str,
    org: Uuid,
    id: &str,
    body: Value,
) -> reqwest::Response {
    reqwest::Client::new()
        .patch(photos_url(app, org, &format!("/{id}")))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the patch call")
}

async fn list_ids(app: &harness::App, query: &str) -> Vec<String> {
    let listed: Value = reqwest::Client::new()
        .get(format!(
            "{}?{query}",
            photos_url(app, app.organization_id, "")
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");
    listed["data"]
        .as_array()
        .unwrap_or_else(|| panic!("the list carries an array: {listed}"))
        .iter()
        .map(|photo| photo["id"].as_str().unwrap().to_owned())
        .collect()
}

async fn cleanup(app: &harness::App) {
    for statement in [
        "DELETE FROM library_photos WHERE org_id = $1",
        "DELETE FROM does WHERE org_id = $1",
        "DELETE FROM doe_models WHERE org_id = $1",
        "DELETE FROM doe_settings WHERE org_id = $1",
    ] {
        sqlx::query(statement)
            .bind(app.organization_id)
            .execute(&app.pool)
            .await
            .unwrap_or_else(|e| panic!("photo library cleanup failed on `{statement}`: {e}"));
    }
    app.cleanup().await;
}

struct SeededDoe {
    id: String,
    project_id: Uuid,
}

async fn seed_doe_without_photo_section(app: &harness::App) -> SeededDoe {
    let project_id = Uuid::now_v7();
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, 'Avenue du 8 mai')")
        .bind(project_id)
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .expect("seed the project");

    let model_id = Uuid::now_v7();
    let sections = json!([
        {"title": "Végétaux", "enabled": true, "kind": {"kind": "supplies", "maker_label": "nursery"}},
    ]);
    sqlx::query(
        "INSERT INTO doe_models (id, org_id, name, sections) VALUES ($1, $2, 'Sans photos', $3)",
    )
    .bind(model_id)
    .bind(app.organization_id)
    .bind(sections)
    .execute(&app.pool)
    .await
    .expect("seed the model");

    let created = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/organizations/{}/projects/{project_id}/doe",
            app.base_url, app.organization_id
        ))
        .bearer_auth(&app.token)
        .json(&json!({ "model_id": model_id }))
        .send()
        .await
        .expect("the api answers the create call");
    assert_eq!(created.status(), 201);
    let created: Value = created.json().await.expect("json");

    SeededDoe {
        id: created["data"]["id"].as_str().unwrap().to_owned(),
        project_id,
    }
}

async fn get_doe(app: &harness::App, doe_id: &str) -> Value {
    let doe: Value = reqwest::Client::new()
        .get(doe_url(app, app.organization_id, doe_id, ""))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call")
        .json()
        .await
        .expect("json");
    doe["data"].clone()
}

async fn place(
    app: &harness::App,
    token: &str,
    org: Uuid,
    doe_id: &str,
    placements: Value,
) -> reqwest::Response {
    reqwest::Client::new()
        .post(doe_url(app, org, doe_id, "/photos/place"))
        .bearer_auth(token)
        .json(&json!({ "placements": placements }))
        .send()
        .await
        .expect("the api answers the place call")
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn spec_photo_2_3_4_a_large_image_is_reduced_once_and_bad_files_are_refused() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let first = upload(&app, &app.token, app.organization_id, jpeg(1, 2600, 1300)).await;
    assert_eq!(first.status(), 201);
    let first: Value = first.json().await.expect("json");
    let photo = &first["data"];
    assert_eq!(photo["width"], 2400);
    assert_eq!(photo["height"], 1200);
    assert_eq!(photo["mime_type"], "image/jpeg");
    assert!(photo["caption"].is_null());
    assert_eq!(photo["tags"], json!([]));
    let id = photo["id"].as_str().unwrap().to_owned();

    let again = upload(&app, &app.token, app.organization_id, jpeg(1, 2600, 1300)).await;
    assert_eq!(again.status(), 200);
    let again: Value = again.json().await.expect("json");
    assert_eq!(again["data"]["id"], id.as_str());

    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM library_photos WHERE org_id = $1")
        .bind(app.organization_id)
        .fetch_one(&app.pool)
        .await
        .expect("count the rows");
    assert_eq!(rows, 1);

    let thumbnail = client
        .get(photos_url(
            &app,
            app.organization_id,
            &format!("/{id}/thumbnail"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("thumbnail");
    assert_eq!(thumbnail.status(), 200);
    assert_eq!(thumbnail.headers()["content-type"], "image/jpeg");
    let bytes = thumbnail.bytes().await.expect("bytes");
    let decoded = image::load_from_memory(&bytes).expect("the thumbnail is an image");
    assert_eq!((decoded.width(), decoded.height()), (400, 200));

    let pdf = upload(
        &app,
        &app.token,
        app.organization_id,
        b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n".to_vec(),
    )
    .await;
    assert_eq!(pdf.status(), 422);

    let empty = upload(&app, &app.token, app.organization_id, Vec::new()).await;
    assert_eq!(empty.status(), 422);

    let valid = jpeg(2, 400, 300);
    let truncated = upload(
        &app,
        &app.token,
        app.organization_id,
        valid[..valid.len() / 3].to_vec(),
    )
    .await;
    assert_eq!(truncated.status(), 422);

    let oversized = upload(
        &app,
        &app.token,
        app.organization_id,
        vec![0u8; MAX_UPLOAD_BYTES + 1024],
    )
    .await;
    assert_eq!(oversized.status(), 413);

    let anonymous = client
        .post(photos_url(&app, app.organization_id, ""))
        .body(jpeg(3, 10, 10))
        .send()
        .await
        .expect("anonymous");
    assert_eq!(anonymous.status(), 401);

    let forbidden = upload(
        &app,
        &app.no_permission_token,
        app.organization_id,
        jpeg(4, 10, 10),
    )
    .await;
    assert_eq!(forbidden.status(), 403);

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn spec_photo_5_6_7_filters_tags_and_isolation() {
    let app = harness::start().await;
    let intruder = seed_intruder(&app.pool).await;
    let client = reqwest::Client::new();

    let roses = upload_new(&app, 10).await["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let terrace = upload_new(&app, 11).await["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let raw = upload_new(&app, 12).await["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let updated = patch(
        &app,
        &app.token,
        app.organization_id,
        &roses,
        json!({ "caption": "Massif de ROSIERS", "tags": [" Rosier ", "terrasse", "ROSIER"] }),
    )
    .await;
    assert_eq!(updated.status(), 200);
    let updated: Value = updated.json().await.expect("json");
    assert_eq!(updated["data"]["tags"], json!(["rosier", "terrasse"]));
    patch(
        &app,
        &app.token,
        app.organization_id,
        &terrace,
        json!({ "tags": ["terrasse"] }),
    )
    .await;

    assert_eq!(list_ids(&app, "to_sort=true").await, vec![raw.clone()]);
    assert_eq!(
        list_ids(&app, "tag=terrasse&tag=rosier").await,
        vec![roses.clone()]
    );
    assert_eq!(list_ids(&app, "q=rosiers").await, vec![roses.clone()]);
    let mut by_tag_text = list_ids(&app, "q=TERRASSE").await;
    by_tag_text.sort();
    let mut expected = vec![roses.clone(), terrace.clone()];
    expected.sort();
    assert_eq!(by_tag_text, expected);
    assert_eq!(list_ids(&app, "").await.len(), 3);

    let cleared = patch(
        &app,
        &app.token,
        app.organization_id,
        &roses,
        json!({ "caption": "" }),
    )
    .await;
    let cleared: Value = cleared.json().await.expect("json");
    assert!(cleared["data"]["caption"].is_null());
    assert_eq!(cleared["data"]["tags"], json!(["rosier", "terrasse"]));

    for bad in [
        json!({ "tags": [""] }),
        json!({ "tags": ["x".repeat(41)] }),
        json!({ "tags": (0..21).map(|n| format!("tag{n}")).collect::<Vec<_>>() }),
    ] {
        let refused = patch(&app, &app.token, app.organization_id, &roses, bad).await;
        assert_eq!(refused.status(), 422);
    }

    let page: Value = client
        .get(format!(
            "{}?per_page=2&page=2",
            photos_url(&app, app.organization_id, "")
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("page")
        .json()
        .await
        .expect("json");
    assert_eq!(page["data"].as_array().unwrap().len(), 1);
    assert_eq!(page["pagination"]["total"], 3);

    let own_org = intruder.organization_id;
    for response in [
        client
            .get(photos_url(&app, own_org, &format!("/{roses}")))
            .bearer_auth(&intruder.token)
            .send()
            .await
            .expect("read"),
        client
            .get(photos_url(&app, own_org, &format!("/{roses}/thumbnail")))
            .bearer_auth(&intruder.token)
            .send()
            .await
            .expect("thumbnail"),
        patch(
            &app,
            &intruder.token,
            own_org,
            &roses,
            json!({ "caption": "pirate" }),
        )
        .await,
        client
            .delete(photos_url(&app, own_org, &format!("/{roses}")))
            .bearer_auth(&intruder.token)
            .send()
            .await
            .expect("delete"),
    ] {
        assert_eq!(response.status(), 404);
    }
    let outside = client
        .get(photos_url(&app, app.organization_id, &format!("/{roses}")))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("outside");
    assert_eq!(outside.status(), 403);
    assert_eq!(list_ids(&app, "").await.len(), 3);

    let project_id = Uuid::now_v7();
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, 'Jardin')")
        .bind(project_id)
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .expect("seed the project");
    let linked = patch(
        &app,
        &app.token,
        app.organization_id,
        &roses,
        json!({ "project_id": project_id }),
    )
    .await;
    let linked: Value = linked.json().await.expect("json");
    assert_eq!(linked["data"]["project_id"], project_id.to_string());
    assert_eq!(
        list_ids(&app, &format!("project_id={project_id}")).await,
        vec![roses.clone()]
    );
    let unlinked = patch(
        &app,
        &app.token,
        app.organization_id,
        &roses,
        json!({ "project_id": null }),
    )
    .await;
    let unlinked: Value = unlinked.json().await.expect("json");
    assert!(unlinked["data"]["project_id"].is_null());

    let deleted = client
        .delete(photos_url(&app, app.organization_id, &format!("/{roses}")))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("delete");
    assert_eq!(deleted.status(), 204);
    let gone = client
        .get(photos_url(&app, app.organization_id, &format!("/{roses}")))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("gone");
    assert_eq!(gone.status(), 404);

    intruder.cleanup(&app.pool).await;
    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn spec_photo_8_9_10_12_placement_copies_reports_and_survives_deletion() {
    let app = harness::start().await;
    let intruder = seed_intruder(&app.pool).await;
    let client = reqwest::Client::new();
    let doe = seed_doe_without_photo_section(&app).await;

    let mut photos = Vec::new();
    for seed in [20, 21, 22] {
        photos.push(upload_new(&app, seed).await["data"].clone());
    }
    let ids: Vec<String> = photos
        .iter()
        .map(|photo| photo["id"].as_str().unwrap().to_owned())
        .collect();

    let placed = place(
        &app,
        &app.token,
        app.organization_id,
        &doe.id,
        json!(
            ids.iter()
                .map(|id| json!({ "photo_id": id, "target": "section", "caption": "Réception" }))
                .collect::<Vec<_>>()
        ),
    )
    .await;
    assert_eq!(placed.status(), 200);
    let placed: Value = placed.json().await.expect("json");
    let sections = placed["data"]["doe"]["sections"].as_array().unwrap();
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[1]["title"], "Photos générales du chantier");
    let files = sections[1]["content"]["photos"].as_array().unwrap();
    assert_eq!(files.len(), 3);
    for file in files {
        assert!(photos.iter().all(|photo| photo["file_key"] != file["key"]));
        assert_eq!(file["caption"], "Réception");
    }
    assert!(
        placed["data"]["results"]
            .as_array()
            .unwrap()
            .iter()
            .all(|result| result["status"] == "placed" && result["key"].is_string())
    );

    let before = get_doe(&app, &doe.id).await;
    let pdf_before = client
        .get(doe_url(&app, app.organization_id, &doe.id, "/pdf"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("pdf");
    assert_eq!(pdf_before.status(), 200);

    for id in &ids {
        let removed = client
            .delete(photos_url(&app, app.organization_id, &format!("/{id}")))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("delete");
        assert_eq!(removed.status(), 204);
    }
    assert_eq!(get_doe(&app, &doe.id).await, before);
    let pdf_after = client
        .get(doe_url(&app, app.organization_id, &doe.id, "/pdf"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("pdf");
    assert_eq!(pdf_after.status(), 200);

    let fresh = upload_new(&app, 30).await["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let another = upload_new(&app, 31).await["data"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let unknown = Uuid::now_v7().to_string();
    let mixed = place(
        &app,
        &app.token,
        app.organization_id,
        &doe.id,
        json!([
            { "photo_id": fresh, "target": "section" },
            { "photo_id": unknown, "target": "section" },
            { "photo_id": another, "target": "cover" },
        ]),
    )
    .await;
    assert_eq!(mixed.status(), 200);
    let mixed: Value = mixed.json().await.expect("json");
    let results = mixed["data"]["results"].as_array().unwrap();
    assert_eq!(results[0]["status"], "placed");
    assert_eq!(results[1]["status"], "failed");
    assert!(results[1]["error"].is_string());
    assert_eq!(results[2]["status"], "placed");
    assert_eq!(mixed["data"]["doe"]["cover_photo_key"], results[2]["key"]);
    let section_files = mixed["data"]["doe"]["sections"][1]["content"]["photos"]
        .as_array()
        .unwrap();
    assert_eq!(section_files.len(), 4);

    let foreign_photo = place(
        &app,
        &intruder.token,
        intruder.organization_id,
        &doe.id,
        json!([{ "photo_id": fresh, "target": "section" }]),
    )
    .await;
    assert_eq!(foreign_photo.status(), 404);

    let too_many: Vec<Value> = (0..51)
        .map(|_| json!({ "photo_id": fresh, "target": "cover" }))
        .collect();
    let refused = place(
        &app,
        &app.token,
        app.organization_id,
        &doe.id,
        json!(too_many),
    )
    .await;
    assert_eq!(refused.status(), 422);

    let _ = doe.project_id;
    intruder.cleanup(&app.pool).await;
    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn spec_photo_11_the_file_ceiling_reports_the_surplus_as_failed() {
    let app = harness::start().await;
    let doe = seed_doe_without_photo_section(&app).await;

    let existing: Vec<Value> = (0..299)
        .map(|n| json!({ "key": format!("doe/seeded-{n}"), "caption": null }))
        .collect();
    let sections = json!([{
        "id": Uuid::now_v7(),
        "title": "Photos",
        "enabled": true,
        "content": { "kind": "photos", "per_page": "two", "photos": existing },
    }]);
    sqlx::query("UPDATE does SET sections = $2 WHERE id = $1")
        .bind(doe.id.parse::<Uuid>().unwrap())
        .bind(sections)
        .execute(&app.pool)
        .await
        .expect("fill the DOE");

    let mut ids = Vec::new();
    for seed in [40, 41, 42] {
        ids.push(
            upload_new(&app, seed).await["data"]["id"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    let placed = place(
        &app,
        &app.token,
        app.organization_id,
        &doe.id,
        json!(
            ids.iter()
                .map(|id| json!({ "photo_id": id, "target": "section" }))
                .collect::<Vec<_>>()
        ),
    )
    .await;
    assert_eq!(placed.status(), 200);
    let placed: Value = placed.json().await.expect("json");

    let statuses: Vec<&str> = placed["data"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| result["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, vec!["placed", "failed", "failed"]);
    assert_eq!(
        placed["data"]["doe"]["sections"][0]["content"]["photos"]
            .as_array()
            .unwrap()
            .len(),
        300
    );

    cleanup(&app).await;
}

async fn copy(app: &harness::App, token: &str, org: Uuid, id: &str) -> reqwest::Response {
    reqwest::Client::new()
        .post(photos_url(app, org, &format!("/{id}/copy")))
        .bearer_auth(token)
        .send()
        .await
        .expect("the api answers the copy call")
}

async fn images_in_preview(app: &harness::App, key: &str) -> usize {
    let style = json!({ "decorations": [{
        "kind": "image", "key": key, "x": 20, "y": 20, "width": 60, "height": 40,
        "rotation": 30, "flip_x": true,
    }] });
    let pages: Value = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/organizations/{}/doe-style/preview/pages",
            app.base_url, app.organization_id
        ))
        .bearer_auth(&app.token)
        .json(&json!({ "style": style }))
        .send()
        .await
        .expect("preview pages")
        .json()
        .await
        .expect("json");
    pages["data"]["pages"]
        .as_array()
        .expect("pages")
        .iter()
        .map(|page| page.as_str().unwrap().matches("<image").count())
        .sum()
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn spec_layout_editor_2_6_a_copied_photo_outlives_the_library() {
    let app = harness::start().await;
    let intruder = seed_intruder(&app.pool).await;
    let client = reqwest::Client::new();
    let photo = upload_new(&app, 40).await["data"].clone();
    let id = photo["id"].as_str().unwrap();

    let copied = copy(&app, &app.token, app.organization_id, id).await;
    assert_eq!(copied.status(), 200);
    let key = copied.json::<Value>().await.expect("json")["data"]["key"]
        .as_str()
        .expect("a key")
        .to_owned();
    assert_ne!(key, photo["file_key"].as_str().unwrap());

    let forbidden = copy(&app, &app.no_permission_token, app.organization_id, id).await;
    assert_eq!(forbidden.status(), 403);
    let foreign = copy(&app, &intruder.token, intruder.organization_id, id).await;
    assert_eq!(foreign.status(), 404);

    let settings = format!(
        "{}/api/v1/organizations/{}/doe-settings",
        app.base_url, app.organization_id
    );
    let saved = client
        .put(&settings)
        .bearer_auth(&app.token)
        .json(&json!({ "style": { "decorations": [{
            "kind": "image", "key": key, "x": 0, "y": 0, "width": 210, "height": 30,
            "rotation": -90, "flip_x": true,
        }] } }))
        .send()
        .await
        .expect("save the style");
    assert_eq!(saved.status(), 200);
    let saved: Value = saved.json().await.expect("json");
    let decoration = &saved["data"]["style"]["decorations"][0];
    assert_eq!(decoration["rotation"], 270);
    assert_eq!(decoration["flip_x"], true);
    assert_eq!(decoration["flip_y"], false);

    let not_a_number = client
        .put(&settings)
        .bearer_auth(&app.token)
        .json(&json!({ "style": { "decorations": [{
            "kind": "rectangle", "fill": "#ff0000", "x": 0, "y": 0, "width": 10, "height": 10,
            "rotation": "quart",
        }] } }))
        .send()
        .await
        .expect("refused style");
    assert!(not_a_number.status().is_client_error());

    let missing = images_in_preview(&app, "doe-style/missing").await;
    let removed = client
        .delete(photos_url(&app, app.organization_id, &format!("/{id}")))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("delete");
    assert_eq!(removed.status(), 204);
    assert!(images_in_preview(&app, &key).await > missing);

    intruder.cleanup(&app.pool).await;
    cleanup(&app).await;
}
