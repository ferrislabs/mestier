use serde_json::{Value, json};
use uuid::Uuid;

use crate::{harness, issuer};

const ADMIN_ROLE: &str = "mestier:admin";

struct Account {
    user_id: Uuid,
    sub: String,
    email: String,
}

impl Account {
    fn token(&self, roles: &[&str]) -> String {
        issuer::mint_with_roles(&self.sub, &self.email, roles)
    }
}

async fn seed_account(app: &harness::App, display_name: &str) -> Account {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-backoffice-{user_id}");
    let email = format!("backoffice-{user_id}@example.com");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(&email)
    .bind(format!("backoffice-{user_id}"))
    .bind(display_name)
    .bind(&sub)
    .execute(&app.pool)
    .await
    .expect("seed the back-office account");
    Account {
        user_id,
        sub,
        email,
    }
}

async fn seed_organization(app: &harness::App, owner: &Account, name: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(name)
        .bind(format!("org-{id}"))
        .bind(owner.user_id)
        .execute(&app.pool)
        .await
        .expect("seed the organization");
    id
}

async fn cleanup(app: &harness::App, organizations: &[Uuid], accounts: &[&Account]) {
    for id in organizations {
        for statement in [
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            sqlx::query(statement)
                .bind(id)
                .execute(&app.pool)
                .await
                .unwrap_or_else(|e| panic!("cleanup failed on `{statement}`: {e}"));
        }
    }
    for account in accounts {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(account.user_id)
            .execute(&app.pool)
            .await
            .expect("clear the back-office account");
    }
}

fn url(app: &harness::App, suffix: &str) -> String {
    format!("{}/api/v1{suffix}", app.base_url)
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
    request.send().await.expect("the api answers the call")
}

async fn set_feature(
    app: &harness::App,
    token: &str,
    organization: Uuid,
    flag: &str,
    enabled: bool,
) -> reqwest::Response {
    call(
        reqwest::Method::PUT,
        app,
        token,
        &format!("/admin/organizations/{organization}/features/{flag}"),
        Some(&json!({ "enabled": enabled })),
    )
    .await
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn only_a_platform_admin_on_their_own_token_changes_a_flag() {
    let app = harness::start().await;
    let staff = seed_account(&app, "Staff").await;
    let outsider = seed_account(&app, "Outsider").await;
    let organization = seed_organization(&app, &staff, "Org flags").await;
    let path = format!("/admin/organizations/{organization}/features/drive_ai");
    let delegated = issuer::mint_delegated_with_roles(&staff.sub, &staff.email, &[ADMIN_ROLE]);

    let anonymous = reqwest::Client::new()
        .put(url(&app, &path))
        .json(&json!({ "enabled": true }))
        .send()
        .await
        .unwrap();
    let without_role =
        set_feature(&app, &outsider.token(&[]), organization, "drive_ai", true).await;
    let client_like = set_feature(&app, &delegated, organization, "drive_ai", true).await;

    assert_eq!(anonymous.status(), 401);
    assert_eq!(without_role.status(), 403);
    assert_eq!(client_like.status(), 403);
    let enabled: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM organization_feature_flags WHERE org_id = $1")
            .bind(organization)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(enabled, 0);
    for suffix in [
        "/admin/stats",
        "/admin/features",
        "/admin/organizations",
        "/admin/users",
    ] {
        let response = call(
            reqwest::Method::GET,
            &app,
            &outsider.token(&[]),
            suffix,
            None,
        )
        .await;
        assert_eq!(response.status(), 403, "{suffix}");
    }

    cleanup(&app, &[organization], &[&staff, &outsider]).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_unknown_flag_is_refused_with_422() {
    let app = harness::start().await;
    let staff = seed_account(&app, "Staff").await;
    let organization = seed_organization(&app, &staff, "Org inconnu").await;

    let response = set_feature(
        &app,
        &staff.token(&[ADMIN_ROLE]),
        organization,
        "inconnu",
        true,
    )
    .await;

    assert_eq!(response.status(), 422);
    cleanup(&app, &[organization], &[&staff]).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_flag_round_trip_is_idempotent_logged_and_seen_by_members_of_that_organization_only() {
    let app = harness::start().await;
    let staff = seed_account(&app, "Staff Mestier").await;
    let member = seed_account(&app, "Membre").await;
    let token = staff.token(&[ADMIN_ROLE]);
    let a = seed_organization(&app, &member, "Treyve Paysages").await;
    let b = seed_organization(&app, &member, "Autre").await;
    for org in [a, b] {
        sqlx::query(
            "INSERT INTO organization_members (organization_id, user_id, last_name) VALUES ($1, $2, 'Membre')",
        )
        .bind(org)
        .bind(member.user_id)
        .execute(&app.pool)
        .await
        .unwrap();
    }
    let member_token = member.token(&[]);
    let features = |org: Uuid| format!("/organizations/{org}/features");

    let read = |org: Uuid| {
        let (app, token, path) = (&app, member_token.clone(), features(org));
        async move {
            let response = call(reqwest::Method::GET, app, &token, &path, None).await;
            assert_eq!(response.status(), 200);
            let json: Value = response.json().await.unwrap();
            json["data"]["features"].clone()
        }
    };
    assert_eq!(read(a).await, json!([]));

    let first = set_feature(&app, &token, a, "drive_ai", true).await;
    assert_eq!(first.status(), 200);
    let first: Value = first.json().await.unwrap();
    assert_eq!(
        first["data"],
        json!({ "key": "drive_ai", "enabled": true, "changed": true })
    );
    let again: Value = set_feature(&app, &token, a, "drive_ai", true)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(again["data"]["changed"], false);
    assert_eq!(read(a).await, json!(["drive_ai"]));
    assert_eq!(read(b).await, json!([]));
    let stranger = seed_account(&app, "Etranger").await;
    let refused = call(
        reqwest::Method::GET,
        &app,
        &stranger.token(&[]),
        &features(a),
        None,
    )
    .await;
    assert_eq!(refused.status(), 403);

    let off: Value = set_feature(&app, &token, a, "drive_ai", false)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(off["data"]["changed"], true);
    let off_again: Value = set_feature(&app, &token, a, "drive_ai", false)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(off_again["data"]["changed"], false);
    assert_eq!(read(a).await, json!([]));

    let detail: Value = call(
        reqwest::Method::GET,
        &app,
        &token,
        &format!("/admin/organizations/{a}"),
        None,
    )
    .await
    .json()
    .await
    .unwrap();
    let data = &detail["data"];
    assert_eq!(data["name"], "Treyve Paysages");
    assert_eq!(data["members"].as_array().unwrap().len(), 1);
    assert_eq!(data["features"][0]["key"], "drive_ai");
    assert_eq!(data["features"][0]["enabled"], false);
    let history = data["history"].as_array().unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0]["enabled"], false);
    assert_eq!(history[1]["enabled"], true);
    let staff_name: String = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
        .bind(staff.user_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(history[0]["actor_name"], staff_name.as_str());

    let found: Value = call(
        reqwest::Method::GET,
        &app,
        &token,
        "/admin/organizations?search=TREYVE%20PAYSAGES&per_page=1",
        None,
    )
    .await
    .json()
    .await
    .unwrap();
    assert!(!found["data"].as_array().unwrap().is_empty());
    assert_eq!(found["pagination"]["per_page"], 1);
    let users: Value = call(
        reqwest::Method::GET,
        &app,
        &token,
        &format!(
            "/admin/users?search={}",
            format!("BACKOFFICE-{}", staff.user_id).to_uppercase()
        ),
        None,
    )
    .await
    .json()
    .await
    .unwrap();
    assert!(
        users["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|user| user["email"] == staff.email)
    );
    let stats = call(reqwest::Method::GET, &app, &token, "/admin/stats", None).await;
    assert_eq!(stats.status(), 200);
    let stats: Value = stats.json().await.unwrap();
    assert_eq!(
        stats["data"]["signups_per_week"].as_array().unwrap().len(),
        12
    );
    let catalogue: Value = call(reqwest::Method::GET, &app, &token, "/admin/features", None)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(catalogue["data"][0]["key"], "drive_ai");

    cleanup(&app, &[a, b], &[&staff, &member, &stranger]).await;
    app.cleanup().await;
}
