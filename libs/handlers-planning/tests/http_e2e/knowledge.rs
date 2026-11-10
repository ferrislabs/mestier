use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    harness,
    photo_library::{Intruder, seed_intruder},
};

fn articles_url(app: &harness::App, organization_id: Uuid, suffix: &str) -> String {
    format!(
        "{}/api/v1/organizations/{organization_id}/knowledge{suffix}",
        app.base_url
    )
}

async fn create_note(app: &harness::App, token: &str, body: Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(articles_url(app, app.organization_id, "/notes"))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the note call")
}

async fn upload(
    app: &harness::App,
    token: &str,
    organization_id: Uuid,
    query: &str,
    mime: &str,
    body: Vec<u8>,
) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!(
            "{}?{query}",
            articles_url(app, organization_id, "/articles")
        ))
        .bearer_auth(token)
        .header("content-type", mime)
        .body(body)
        .send()
        .await
        .expect("the api answers the upload call")
}

async fn search(app: &harness::App, query: &str) -> Vec<String> {
    let listed: Value = reqwest::Client::new()
        .get(format!(
            "{}?{query}",
            articles_url(app, app.organization_id, "/articles")
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
        .map(|article| article["id"].as_str().unwrap().to_owned())
        .collect()
}

async fn seed_project(app: &harness::App, organization_id: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, 'Avenue du 8 mai')")
        .bind(id)
        .bind(organization_id)
        .execute(&app.pool)
        .await
        .expect("seed the project");
    id
}

async fn cleanup(app: &harness::App) {
    sqlx::query("DELETE FROM knowledge_articles WHERE org_id = $1")
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .expect("knowledge cleanup");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn a_note_is_created_found_edited_and_deleted() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let created = create_note(
        &app,
        &app.token,
        json!({"title": "  Conditions de paiement ", "body": "30 jours fin de mois", "tags": ["CGV", "cgv"]}),
    )
    .await;
    assert_eq!(created.status(), 201);
    let created: Value = created.json().await.unwrap();
    let id = created["data"]["id"].as_str().unwrap().to_owned();
    assert_eq!(created["data"]["title"], "Conditions de paiement");
    assert_eq!(created["data"]["tags"], json!(["cgv"]));

    assert_eq!(search(&app, "q=paiement").await, vec![id.clone()]);
    assert_eq!(search(&app, "tag=cgv").await, vec![id.clone()]);
    assert!(search(&app, "q=introuvable").await.is_empty());

    let patched = client
        .patch(articles_url(
            &app,
            app.organization_id,
            &format!("/articles/{id}"),
        ))
        .bearer_auth(&app.token)
        .json(&json!({"body": "45 jours", "tags": ["revu"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(patched.status(), 200);
    assert!(search(&app, "tag=cgv").await.is_empty());

    let empty = create_note(&app, &app.token, json!({"title": "Vide", "body": "  "})).await;
    assert_eq!(empty.status(), 422);

    let deleted = client
        .delete(articles_url(
            &app,
            app.organization_id,
            &format!("/articles/{id}"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap();
    assert_eq!(deleted.status(), 204);
    assert!(search(&app, "").await.is_empty());

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn a_text_file_is_searchable_by_its_content_and_downloads_unchanged() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let content = "Tarif gazon en rouleau: 6 euros le metre carre"
        .as_bytes()
        .to_vec();

    let uploaded = upload(
        &app,
        &app.token,
        app.organization_id,
        "title=Tarifs%20fournisseur&tag=tarif",
        "text/plain",
        content.clone(),
    )
    .await;
    assert_eq!(uploaded.status(), 201);
    let uploaded: Value = uploaded.json().await.unwrap();
    let id = uploaded["data"]["id"].as_str().unwrap().to_owned();
    assert_eq!(uploaded["data"]["kind"], "file");
    assert!(
        uploaded["data"].get("file_key").is_none() && uploaded["data"]["file"].get("key").is_none(),
        "the storage key never leaves the api"
    );

    assert_eq!(search(&app, "q=gazon").await, vec![id.clone()]);

    let one: Value = client
        .get(articles_url(
            &app,
            app.organization_id,
            &format!("/articles/{id}"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        one["data"]["extracted_text"]
            .as_str()
            .unwrap()
            .contains("6 euros")
    );
    let listed: Value = client
        .get(articles_url(&app, app.organization_id, "/articles"))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(listed["data"][0].get("extracted_text").is_none());

    let download = client
        .get(articles_url(
            &app,
            app.organization_id,
            &format!("/articles/{id}/download"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap();
    assert_eq!(download.status(), 200);
    assert_eq!(download.headers()["x-content-type-options"], "nosniff");
    assert_eq!(download.bytes().await.unwrap().to_vec(), content);

    let refused = upload(
        &app,
        &app.token,
        app.organization_id,
        "title=Archive",
        "application/zip",
        b"PK".to_vec(),
    )
    .await;
    assert_eq!(refused.status(), 422);
    let untitled = upload(
        &app,
        &app.token,
        app.organization_id,
        "tag=x",
        "text/plain",
        b"x".to_vec(),
    )
    .await;
    assert_eq!(untitled.status(), 400);

    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn an_article_is_linked_to_a_project_found_through_it_and_never_to_a_foreign_one() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let intruder: Intruder = seed_intruder(&app.pool).await;
    let project = seed_project(&app, app.organization_id).await;
    let foreign_project = seed_project(&app, intruder.organization_id).await;

    let created: Value = create_note(&app, &app.token, json!({"title": "Devis", "body": "x"}))
        .await
        .json()
        .await
        .unwrap();
    let id = created["data"]["id"].as_str().unwrap().to_owned();
    let links = articles_url(&app, app.organization_id, &format!("/articles/{id}/links"));

    for _ in 0..2 {
        let linked = client
            .post(&links)
            .bearer_auth(&app.token)
            .json(&json!({"resource_type": "project", "resource_id": project}))
            .send()
            .await
            .unwrap();
        assert_eq!(linked.status(), 200);
    }
    assert_eq!(
        search(
            &app,
            &format!("resource_type=project&resource_id={project}")
        )
        .await,
        vec![id.clone()]
    );
    let one: Value = client
        .get(articles_url(
            &app,
            app.organization_id,
            &format!("/articles/{id}"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(one["data"]["links"].as_array().unwrap().len(), 1);

    let foreign = client
        .post(&links)
        .bearer_auth(&app.token)
        .json(&json!({"resource_type": "project", "resource_id": foreign_project}))
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), 404);

    let unlinked = client
        .delete(format!("{links}/project/{project}"))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap();
    assert_eq!(unlinked.status(), 204);
    assert!(
        search(
            &app,
            &format!("resource_type=project&resource_id={project}")
        )
        .await
        .is_empty()
    );

    sqlx::query("DELETE FROM projects WHERE org_id = $1")
        .bind(intruder.organization_id)
        .execute(&app.pool)
        .await
        .unwrap();
    intruder.cleanup(&app.pool).await;
    cleanup(&app).await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn another_organization_and_a_member_without_the_permission_are_refused() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let intruder: Intruder = seed_intruder(&app.pool).await;

    let created: Value = create_note(
        &app,
        &app.token,
        json!({"title": "Privé", "body": "secret"}),
    )
    .await
    .json()
    .await
    .unwrap();
    let id = created["data"]["id"].as_str().unwrap().to_owned();

    // Another organization's owner, through his own organization's path.
    let theirs = articles_url(&app, intruder.organization_id, &format!("/articles/{id}"));
    for response in [
        client
            .get(&theirs)
            .bearer_auth(&intruder.token)
            .send()
            .await
            .unwrap(),
        client
            .delete(&theirs)
            .bearer_auth(&intruder.token)
            .send()
            .await
            .unwrap(),
    ] {
        assert_eq!(response.status(), 404);
    }
    // Through ours: he is not a member.
    let ours = articles_url(&app, app.organization_id, &format!("/articles/{id}"));
    assert_eq!(
        client
            .get(&ours)
            .bearer_auth(&intruder.token)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );

    // A member holding no role.
    let list = articles_url(&app, app.organization_id, "/articles");
    assert_eq!(
        client
            .get(&list)
            .bearer_auth(&app.no_permission_token)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        create_note(
            &app,
            &app.no_permission_token,
            json!({"title": "X", "body": "y"})
        )
        .await
        .status(),
        403
    );

    intruder.cleanup(&app.pool).await;
    cleanup(&app).await;
}
