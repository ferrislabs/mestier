use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{harness, issuer};

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

async fn seed_intruder(pool: &PgPool) -> Intruder {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-doe-intruder-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("doe-intruder-{user_id}@example.com"))
    .bind(format!("doe-intruder-{user_id}"))
    .bind("Intruder")
    .bind(&sub)
    .execute(pool)
    .await
    .expect("seed the intruder user");

    let organization_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id)
        .bind("Intruder Org")
        .bind(format!("doe-intruder-org-{organization_id}"))
        .bind(user_id)
        .execute(pool)
        .await
        .expect("seed the intruder's organization");
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(user_id)
    .bind("Intruder")
    .execute(pool)
    .await
    .expect("seed the intruder's membership");

    Intruder {
        token: issuer::mint(&sub),
        organization_id,
        user_id,
    }
}

struct Seeded {
    project_id: Uuid,
    model_id: Uuid,
}

async fn seed_project_and_model(app: &harness::App) -> Seeded {
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
        {"title": "Photos", "enabled": true, "kind": {"kind": "photos", "per_page": "two"}},
        {"title": "Jeux", "enabled": false, "kind": {"kind": "supplies", "maker_label": "manufacturer"}},
    ]);
    sqlx::query(
        "INSERT INTO doe_models (id, org_id, name, sections) VALUES ($1, $2, 'Paysagiste', $3)",
    )
    .bind(model_id)
    .bind(app.organization_id)
    .bind(sections)
    .execute(&app.pool)
    .await
    .expect("seed the model");

    Seeded {
        project_id,
        model_id,
    }
}

async fn cleanup(app: &harness::App) {
    for statement in [
        "DELETE FROM does WHERE org_id = $1",
        "DELETE FROM doe_models WHERE org_id = $1",
    ] {
        sqlx::query(statement)
            .bind(app.organization_id)
            .execute(&app.pool)
            .await
            .unwrap_or_else(|e| panic!("DOE cleanup failed on `{statement}`: {e}"));
    }
    app.cleanup().await;
}

fn project_doe_url(app: &harness::App, organization_id: Uuid, project_id: Uuid) -> String {
    format!(
        "{}/api/v1/organizations/{organization_id}/projects/{project_id}/doe",
        app.base_url
    )
}

fn doe_url(app: &harness::App, organization_id: Uuid, doe_id: &str, suffix: &str) -> String {
    format!(
        "{}/api/v1/organizations/{organization_id}/does/{doe_id}{suffix}",
        app.base_url
    )
}

async fn create_doe(app: &harness::App, seeded: &Seeded) -> Value {
    let created = reqwest::Client::new()
        .post(project_doe_url(app, app.organization_id, seeded.project_id))
        .bearer_auth(&app.token)
        .json(&json!({ "model_id": seeded.model_id }))
        .send()
        .await
        .expect("the api answers the create call");
    assert_eq!(created.status(), 201);
    created.json().await.expect("the create answer is json")
}

fn put_body(doe: &Value) -> Value {
    let data = &doe["data"];
    json!({
        "title": data["title"],
        "issued_on": data["issued_on"],
        "revision": data["revision"],
        "parties": data["parties"],
        "sections": data["sections"],
    })
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_doe_is_created_once_read_and_replaced() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let seeded = seed_project_and_model(&app).await;

    let missing = client
        .get(project_doe_url(
            &app,
            app.organization_id,
            seeded.project_id,
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call");
    assert_eq!(missing.status(), 404);

    let created = create_doe(&app, &seeded).await;
    assert_eq!(created["data"]["title"], "Avenue du 8 mai");
    assert_eq!(created["data"]["revision"], "A");
    assert_eq!(created["data"]["sections"].as_array().unwrap().len(), 3);
    let doe_id = created["data"]["id"].as_str().unwrap().to_owned();

    let again = client
        .post(project_doe_url(
            &app,
            app.organization_id,
            seeded.project_id,
        ))
        .bearer_auth(&app.token)
        .json(&json!({ "model_id": seeded.model_id }))
        .send()
        .await
        .expect("the api answers the second create call");
    assert_eq!(again.status(), 409);

    let by_project: Value = client
        .get(project_doe_url(
            &app,
            app.organization_id,
            seeded.project_id,
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call")
        .json()
        .await
        .expect("json");
    assert_eq!(by_project["data"]["id"], doe_id);

    let mut body = put_body(&created);
    body["title"] = json!("DOE Avenue");
    body["revision"] = json!("B");
    let replaced = client
        .put(doe_url(&app, app.organization_id, &doe_id, ""))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the put call");
    assert_eq!(replaced.status(), 200);
    let replaced: Value = replaced.json().await.expect("json");
    assert_eq!(replaced["data"]["title"], "DOE Avenue");
    assert_eq!(replaced["data"]["revision"], "B");

    body["revision"] = json!("TOO-LONG-REV");
    let invalid = client
        .put(doe_url(&app, app.organization_id, &doe_id, ""))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the invalid put call");
    assert_eq!(invalid.status(), 409);

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_doe_6_another_organizations_member_gets_404_on_read_write_and_pdf() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let intruder = seed_intruder(&app.pool).await;
    let seeded = seed_project_and_model(&app).await;
    let created = create_doe(&app, &seeded).await;
    let doe_id = created["data"]["id"].as_str().unwrap().to_owned();

    let read = client
        .get(doe_url(&app, intruder.organization_id, &doe_id, ""))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("read");
    assert_eq!(read.status(), 404);

    let write = client
        .put(doe_url(&app, intruder.organization_id, &doe_id, ""))
        .bearer_auth(&intruder.token)
        .json(&put_body(&created))
        .send()
        .await
        .expect("write");
    assert_eq!(write.status(), 404);

    let pdf = client
        .get(doe_url(&app, intruder.organization_id, &doe_id, "/pdf"))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("pdf");
    assert_eq!(pdf.status(), 404);

    let by_project = client
        .get(project_doe_url(
            &app,
            intruder.organization_id,
            seeded.project_id,
        ))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("by project");
    assert_eq!(by_project.status(), 404);

    let outside = client
        .get(doe_url(&app, app.organization_id, &doe_id, ""))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("outside");
    assert_eq!(outside.status(), 403);

    intruder.cleanup(&app.pool).await;
    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_doe_7_a_member_without_planning_manage_reads_but_cannot_write() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let seeded = seed_project_and_model(&app).await;
    let created = create_doe(&app, &seeded).await;
    let doe_id = created["data"]["id"].as_str().unwrap().to_owned();

    let write = client
        .put(doe_url(&app, app.organization_id, &doe_id, ""))
        .bearer_auth(&app.no_permission_token)
        .json(&put_body(&created))
        .send()
        .await
        .expect("write");
    assert_eq!(write.status(), 403);

    let draft = client
        .post(doe_url(
            &app,
            app.organization_id,
            &doe_id,
            "/supplies/draft",
        ))
        .bearer_auth(&app.no_permission_token)
        .json(&json!({ "product_id": Uuid::now_v7() }))
        .send()
        .await
        .expect("draft");
    assert_eq!(draft.status(), 403);

    let other_project = Uuid::now_v7();
    let create = client
        .post(project_doe_url(&app, app.organization_id, other_project))
        .bearer_auth(&app.no_permission_token)
        .json(&json!({ "model_id": seeded.model_id }))
        .send()
        .await
        .expect("create");
    assert_eq!(create.status(), 403);

    let read = client
        .get(doe_url(&app, app.organization_id, &doe_id, ""))
        .bearer_auth(&app.no_permission_token)
        .send()
        .await
        .expect("read");
    assert_eq!(read.status(), 200);

    let pdf = client
        .get(doe_url(&app, app.organization_id, &doe_id, "/pdf"))
        .bearer_auth(&app.no_permission_token)
        .send()
        .await
        .expect("pdf");
    assert_eq!(pdf.status(), 200);
    assert_eq!(
        pdf.headers()["content-type"].to_str().unwrap(),
        "application/pdf"
    );
    let disposition = pdf.headers()["content-disposition"]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(
        disposition.contains("filename=\"DOE - Avenue du 8 mai - A.pdf\""),
        "{disposition}"
    );
    let bytes = pdf.bytes().await.expect("pdf bytes");
    assert!(bytes.starts_with(b"%PDF"));

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and the configured file storage"]
async fn spec_doe_10_a_file_missing_from_storage_is_a_409_naming_it() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let seeded = seed_project_and_model(&app).await;
    let created = create_doe(&app, &seeded).await;
    let doe_id = created["data"]["id"].as_str().unwrap().to_owned();

    let mut body = put_body(&created);
    body["sections"][1]["content"]["photos"] =
        json!([{ "key": "doe-e2e/never-uploaded.jpg", "caption": "Massif" }]);
    let replaced = client
        .put(doe_url(&app, app.organization_id, &doe_id, ""))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("put");
    assert_eq!(replaced.status(), 200);

    let pdf = client
        .get(doe_url(&app, app.organization_id, &doe_id, "/pdf"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("pdf");
    assert_eq!(pdf.status(), 409);
    let message: Value = pdf.json().await.expect("json");
    assert!(
        message["message"]
            .as_str()
            .unwrap()
            .contains("doe-e2e/never-uploaded.jpg"),
        "{message}"
    );

    cleanup(&app).await;
}

fn doe_model_url(app: &harness::App, organization_id: Uuid, model_id: Uuid) -> String {
    format!(
        "{}/api/v1/organizations/{organization_id}/doe-models/{model_id}",
        app.base_url
    )
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_doe_model_stays_inside_its_organization_and_writes_need_reference_manage() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let seeded = seed_project_and_model(&app).await;
    let intruder = seed_intruder(&app.pool).await;
    let body = json!({
        "name": "Électricité",
        "description": null,
        "sections": [{ "title": "Plans", "enabled": true, "kind": { "kind": "plans" } }],
    });

    for (method, expected) in [("GET", 404), ("PUT", 404)] {
        let request = match method {
            "GET" => client.get(doe_model_url(
                &app,
                intruder.organization_id,
                seeded.model_id,
            )),
            _ => client
                .put(doe_model_url(
                    &app,
                    intruder.organization_id,
                    seeded.model_id,
                ))
                .json(&body),
        };
        let response = request
            .bearer_auth(&intruder.token)
            .send()
            .await
            .expect("intruder request");
        assert_eq!(
            response.status(),
            expected,
            "{method} through the intruder's organization"
        );
    }

    let read = client
        .get(doe_model_url(&app, app.organization_id, seeded.model_id))
        .bearer_auth(&app.no_permission_token)
        .send()
        .await
        .expect("read");
    assert_eq!(read.status(), 200);

    let write = client
        .put(doe_model_url(&app, app.organization_id, seeded.model_id))
        .bearer_auth(&app.no_permission_token)
        .json(&body)
        .send()
        .await
        .expect("write");
    assert_eq!(write.status(), 403);

    let unchanged = client
        .get(doe_model_url(&app, app.organization_id, seeded.model_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("read back")
        .json::<Value>()
        .await
        .expect("json");
    assert_eq!(unchanged["data"]["name"], "Paysagiste");

    intruder.cleanup(&app.pool).await;
    cleanup(&app).await;
}
