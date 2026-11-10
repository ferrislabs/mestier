use chrono::{Duration, Utc};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{harness, issuer};

const ADMIN_ROLE: &str = "mestier:admin";

struct Reader {
    user_id: Uuid,
    token: String,
}

async fn seed_reader(app: &harness::App, roles: &[&str]) -> Reader {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-news-{user_id}");
    let email = format!("news-{user_id}@example.com");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(&email)
    .bind(format!("news-{user_id}"))
    .bind("News Reader")
    .bind(&sub)
    .execute(&app.pool)
    .await
    .expect("seed the news reader");
    Reader {
        user_id,
        token: issuer::mint_with_roles(&sub, &email, roles),
    }
}

async fn cleanup(app: &harness::App, readers: &[&Reader], news_ids: &[Uuid]) {
    sqlx::query("DELETE FROM news WHERE id = ANY($1)")
        .bind(news_ids)
        .execute(&app.pool)
        .await
        .expect("clear the news");
    for reader in readers {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(reader.user_id)
            .execute(&app.pool)
            .await
            .expect("clear the news reader");
    }
}

fn url(app: &harness::App, suffix: &str) -> String {
    format!("{}/api/v1{suffix}", app.base_url)
}

fn body(title: &str, published_at: Option<String>) -> Value {
    json!({
        "title": title,
        "image_key": null,
        "items": [{ "icon": "sparkles", "title": "Connecteurs", "text": "Branchez vos outils." }],
        "cta": { "label": "Voir", "url": "https://mestier.app/news" },
        "published_at": published_at,
    })
}

async fn call(
    method: reqwest::Method,
    app: &harness::App,
    token: &str,
    suffix: &str,
    payload: Option<&Value>,
) -> reqwest::Response {
    let request = reqwest::Client::new()
        .request(method, url(app, suffix))
        .bearer_auth(token);
    let request = match payload {
        Some(payload) => request.json(payload),
        None => request,
    };
    request.send().await.expect("the api answers the news call")
}

async fn listed(app: &harness::App, token: &str, suffix: &str) -> Vec<Uuid> {
    let response = call(reqwest::Method::GET, app, token, suffix, None).await;
    assert_eq!(response.status(), 200, "{suffix}");
    let json: Value = response.json().await.expect("the list is json");
    json["data"]
        .as_array()
        .unwrap_or_else(|| panic!("the list carries an array: {json}"))
        .iter()
        .map(|news| news["id"].as_str().unwrap().parse().unwrap())
        .collect()
}

async fn create(app: &harness::App, admin: &Reader, payload: &Value) -> Uuid {
    let response = call(
        reqwest::Method::POST,
        app,
        &admin.token,
        "/admin/news",
        Some(payload),
    )
    .await;
    assert_eq!(response.status(), 201);
    let json: Value = response.json().await.expect("the news is json");
    json["data"]["id"].as_str().unwrap().parse().unwrap()
}

fn past() -> Option<String> {
    Some((Utc::now() - Duration::minutes(5)).to_rfc3339())
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_draft_reaches_nobody_and_a_published_news_is_seen_once_per_user() {
    let app = harness::start().await;
    let admin = seed_reader(&app, &[ADMIN_ROLE]).await;
    let reader = seed_reader(&app, &[]).await;
    let other = seed_reader(&app, &[]).await;

    let id = create(&app, &admin, &body("Brouillon", None)).await;
    assert!(
        !listed(&app, &reader.token, "/news/unseen")
            .await
            .contains(&id)
    );
    assert!(!listed(&app, &reader.token, "/news").await.contains(&id));

    let published = call(
        reqwest::Method::PUT,
        &app,
        &admin.token,
        &format!("/admin/news/{id}"),
        Some(&body("Publiée", past())),
    )
    .await;
    assert_eq!(published.status(), 200);
    let published: Value = published.json().await.unwrap();
    assert_eq!(published["data"]["title"], "Publiée");
    assert_eq!(published["data"]["items"][0]["icon"], "sparkles");
    assert_eq!(published["data"]["cta"]["url"], "https://mestier.app/news");

    assert!(
        listed(&app, &reader.token, "/news/unseen")
            .await
            .contains(&id)
    );

    for _ in 0..2 {
        let seen = call(
            reqwest::Method::POST,
            &app,
            &reader.token,
            "/news/seen",
            Some(&json!({ "ids": [id] })),
        )
        .await;
        assert_eq!(seen.status(), 204);
    }

    assert!(
        !listed(&app, &reader.token, "/news/unseen")
            .await
            .contains(&id)
    );
    assert!(listed(&app, &reader.token, "/news").await.contains(&id));
    assert!(
        listed(&app, &other.token, "/news/unseen")
            .await
            .contains(&id)
    );

    cleanup(&app, &[&admin, &reader, &other], &[id]).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_scheduled_news_is_served_to_nobody_but_listed_in_the_back_office() {
    let app = harness::start().await;
    let admin = seed_reader(&app, &[ADMIN_ROLE]).await;
    let reader = seed_reader(&app, &[]).await;

    let tomorrow = Some((Utc::now() + Duration::days(1)).to_rfc3339());
    let id = create(&app, &admin, &body("Programmée", tomorrow)).await;

    assert!(!listed(&app, &reader.token, "/news").await.contains(&id));
    assert!(
        !listed(&app, &reader.token, "/news/unseen")
            .await
            .contains(&id)
    );
    assert!(
        listed(&app, &admin.token, "/admin/news")
            .await
            .contains(&id)
    );

    cleanup(&app, &[&admin, &reader], &[id]).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn only_a_platform_admin_reaches_the_back_office() {
    let app = harness::start().await;
    let admin = seed_reader(&app, &[ADMIN_ROLE]).await;
    let reader = seed_reader(&app, &[]).await;
    let id = create(&app, &admin, &body("Réservée", None)).await;

    let anonymous = reqwest::Client::new()
        .get(url(&app, "/admin/news"))
        .send()
        .await
        .unwrap();
    assert_eq!(anonymous.status(), 401);

    let payload = body("Intrus", None);
    let item = format!("/admin/news/{id}");
    for (method, suffix, payload) in [
        (reqwest::Method::GET, "/admin/news", None),
        (reqwest::Method::POST, "/admin/news", Some(&payload)),
        (reqwest::Method::PUT, item.as_str(), Some(&payload)),
        (reqwest::Method::DELETE, item.as_str(), None),
    ] {
        let response = call(method.clone(), &app, &reader.token, suffix, payload).await;
        assert_eq!(response.status(), 403, "{method} {suffix}");
    }

    let access = |token: String| {
        let app = &app;
        async move {
            let response = call(reqwest::Method::GET, app, &token, "/news/access", None).await;
            assert_eq!(response.status(), 200);
            let json: Value = response.json().await.unwrap();
            json["data"]["can_manage"].as_bool().unwrap()
        }
    };
    assert!(!access(reader.token.clone()).await);
    assert!(access(admin.token.clone()).await);

    cleanup(&app, &[&admin, &reader], &[id]).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn invalid_content_is_refused_and_nothing_is_stored() {
    let app = harness::start().await;
    let admin = seed_reader(&app, &[ADMIN_ROLE]).await;
    let marker = format!("refusee-{}", Uuid::now_v7());

    let mut empty_title = body("   ", None);
    empty_title["title"] = json!("   ");
    let mut seven_items = body(&marker, None);
    seven_items["items"] = json!(
        (0..7)
            .map(|_| json!({ "icon": "bell", "title": "Point", "text": "Texte" }))
            .collect::<Vec<_>>()
    );
    let mut script_link = body(&marker, None);
    script_link["cta"]["url"] = json!("javascript:alert(1)");

    for payload in [&empty_title, &seven_items, &script_link] {
        let response = call(
            reqwest::Method::POST,
            &app,
            &admin.token,
            "/admin/news",
            Some(payload),
        )
        .await;
        assert_eq!(response.status(), 422, "{payload}");
    }

    let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM news WHERE title = $1")
        .bind(&marker)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(stored, 0);

    cleanup(&app, &[&admin], &[]).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_seen_list_is_bounded_and_ignores_unknown_ids() {
    let app = harness::start().await;
    let reader = seed_reader(&app, &[]).await;

    let none = call(
        reqwest::Method::POST,
        &app,
        &reader.token,
        "/news/seen",
        Some(&json!({ "ids": [] })),
    )
    .await;
    assert_eq!(none.status(), 422);

    let too_many: Vec<Uuid> = (0..51).map(|_| Uuid::now_v7()).collect();
    let many = call(
        reqwest::Method::POST,
        &app,
        &reader.token,
        "/news/seen",
        Some(&json!({ "ids": too_many })),
    )
    .await;
    assert_eq!(many.status(), 422);

    let unknown = call(
        reqwest::Method::POST,
        &app,
        &reader.token,
        "/news/seen",
        Some(&json!({ "ids": [Uuid::now_v7()] })),
    )
    .await;
    assert_eq!(unknown.status(), 204);

    cleanup(&app, &[&reader], &[]).await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn deleting_a_news_removes_it_and_its_views() {
    let app = harness::start().await;
    let admin = seed_reader(&app, &[ADMIN_ROLE]).await;
    let reader = seed_reader(&app, &[]).await;
    let id = create(&app, &admin, &body("Éphémère", past())).await;

    let seen = call(
        reqwest::Method::POST,
        &app,
        &reader.token,
        "/news/seen",
        Some(&json!({ "ids": [id] })),
    )
    .await;
    assert_eq!(seen.status(), 204);
    let views = |app: &harness::App| {
        let pool = app.pool.clone();
        async move {
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM news_views WHERE news_id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap()
        }
    };
    assert_eq!(views(&app).await, 1);

    let item = format!("/admin/news/{id}");
    let deleted = call(reqwest::Method::DELETE, &app, &admin.token, &item, None).await;
    assert_eq!(deleted.status(), 204);

    assert!(!listed(&app, &reader.token, "/news").await.contains(&id));
    assert_eq!(views(&app).await, 0);

    let again = call(reqwest::Method::DELETE, &app, &admin.token, &item, None).await;
    assert_eq!(again.status(), 404);
    let missing = call(
        reqwest::Method::PUT,
        &app,
        &admin.token,
        &item,
        Some(&body("Fantôme", None)),
    )
    .await;
    assert_eq!(missing.status(), 404);

    cleanup(&app, &[&admin, &reader], &[id]).await;
}
