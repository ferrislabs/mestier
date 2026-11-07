use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App};

const MANAGE_REFERENCE: i64 = 1 << 14;

async fn grant_reference_manage(app: &App) {
    let member_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM organization_members WHERE organization_id = $1 AND user_id IS NOT NULL LIMIT 1",
    )
    .bind(app.organization_id)
    .fetch_one(&app.pool)
    .await
    .expect("the owner seat");
    let role_id = Uuid::now_v7();
    sqlx::query("INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, 'Reference', $3)")
        .bind(role_id)
        .bind(app.organization_id)
        .bind(MANAGE_REFERENCE)
        .execute(&app.pool)
        .await
        .expect("seed the role");
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(member_id)
        .bind(role_id)
        .execute(&app.pool)
        .await
        .expect("assign the role");
}

async fn forget(app: &App) {
    for statement in [
        "DELETE FROM suppliers WHERE org_id = $1",
        "DELETE FROM expense_categories WHERE org_id = $1",
        "DELETE FROM member_roles WHERE role_id IN (SELECT id FROM roles WHERE organization_id = $1)",
        "DELETE FROM roles WHERE organization_id = $1",
    ] {
        sqlx::query(statement)
            .bind(app.organization_id)
            .execute(&app.pool)
            .await
            .unwrap_or_else(|error| panic!("cleanup failed on `{statement}`: {error}"));
    }
}

async fn seed_category(app: &App, organization_id: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO expense_categories (id, org_id, name, hue) VALUES ($1, $2, 'Matériaux', 60)",
    )
    .bind(id)
    .bind(organization_id)
    .execute(&app.pool)
    .await
    .expect("seed a category");

    id
}

fn suppliers_url(app: &App, suffix: &str) -> String {
    format!(
        "{}/api/v1/organizations/{}/suppliers{suffix}",
        app.base_url, app.organization_id
    )
}

async fn call(
    app: &App,
    method: reqwest::Method,
    url: String,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request = reqwest::Client::new()
        .request(method, url)
        .bearer_auth(&app.token);
    let request = match body {
        Some(body) => request.json(&body),
        None => request,
    };
    let response = request.send().await.expect("the api answers");
    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

fn full_body(category_id: Uuid) -> Value {
    json!({
        "name": "Point P",
        "registration_number": "732 829 320",
        "iban": "FR76 3000 6000 0112 3456 7890 189",
        "payment_terms_days": 45,
        "default_category_id": category_id,
        "notes": "Livraison le mardi",
        "contacts": [
            { "first_name": "Jean", "last_name": "Bernard", "role": "Commercial", "email": "jean@pointp.fr", "phone": "0240000000", "mobile": "0600000000" },
            { "last_name": "Compta" }
        ],
        "addresses": [
            { "kind": "BILLING", "label": "Siège", "street": "1 rue A", "postal_code": "44000", "city": "Nantes", "country": "France" },
            { "kind": "SITE", "label": "Dépôt", "street": "2 rue B", "line2": "Bât. 3", "postal_code": "44800", "city": "Saint-Herblain", "country": "France" }
        ],
    })
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn spec_crm_6_10_a_supplier_stores_its_profile_replaces_contacts_and_archives() {
    let app = harness::start().await;
    grant_reference_manage(&app).await;
    let category = seed_category(&app, app.organization_id).await;

    let (status, created) = call(
        &app,
        reqwest::Method::POST,
        suppliers_url(&app, ""),
        Some(full_body(category)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let data = &created["data"];
    assert_eq!(data["iban"], json!("FR76 3000 6000 0112 3456 7890 189"));
    assert_eq!(data["payment_terms_days"], json!(45));
    assert_eq!(data["default_category_id"], json!(category));
    assert_eq!(data["notes"], json!("Livraison le mardi"));
    assert_eq!(data["archived_at"], Value::Null);
    assert_eq!(data["contacts"].as_array().map(Vec::len), Some(2));
    assert_eq!(data["contacts"][0]["mobile"], json!("0600000000"));
    assert_eq!(data["addresses"][1]["kind"], json!("SITE"));
    assert_eq!(data["addresses"][1]["line2"], json!("Bât. 3"));
    let id = data["id"].as_str().expect("an id").to_owned();
    let item = suppliers_url(&app, &format!("/{id}"));

    let (status, read) = call(&app, reqwest::Method::GET, item.clone(), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(read["data"]["contacts"][1]["last_name"], json!("Compta"));

    let mut smaller = full_body(category);
    smaller["contacts"] = json!([{ "last_name": "Seul" }]);
    smaller["addresses"] = json!([]);
    smaller["payment_terms_days"] = Value::Null;
    let (status, updated) = call(&app, reqwest::Method::PUT, item.clone(), Some(smaller)).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(
        updated["data"]["contacts"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        updated["data"]["addresses"].as_array().map(Vec::len),
        Some(0)
    );
    assert_eq!(updated["data"]["payment_terms_days"], Value::Null);

    for days in [0, 999] {
        let mut body = full_body(category);
        body["payment_terms_days"] = json!(days);
        let (status, ok) = call(&app, reqwest::Method::PUT, item.clone(), Some(body)).await;
        assert_eq!(status, StatusCode::OK, "{days}: {ok}");
    }
    for days in [1000, -1] {
        let mut body = full_body(category);
        body["payment_terms_days"] = json!(days);
        let (status, refused) = call(&app, reqwest::Method::PUT, item.clone(), Some(body)).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{days}: {refused}"
        );
        assert_eq!(refused["details"]["field"], json!("payment_terms_days"));
    }

    let (status, archived) = call(
        &app,
        reqwest::Method::POST,
        suppliers_url(&app, &format!("/{id}/archive")),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{archived}");
    assert!(archived["data"]["archived_at"].is_string());
    let (_, listed) = call(&app, reqwest::Method::GET, suppliers_url(&app, ""), None).await;
    assert_eq!(listed["data"][0]["id"], json!(id));
    assert!(listed["data"][0]["archived_at"].is_string());
    let (status, restored) = call(
        &app,
        reqwest::Method::POST,
        suppliers_url(&app, &format!("/{id}/restore")),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{restored}");
    assert_eq!(restored["data"]["archived_at"], Value::Null);

    forget(&app).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn a_default_category_of_another_organization_is_not_found_and_a_write_without_role_is_refused()
 {
    let app = harness::start().await;
    let other_user = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, 'Other', $4)",
    )
    .bind(other_user)
    .bind(format!("other-{other_user}@example.com"))
    .bind(format!("other-{other_user}"))
    .bind(format!("sub-other-{other_user}"))
    .execute(&app.pool)
    .await
    .expect("seed the other user");
    let other_org = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, 'Other', $2, $3)",
    )
    .bind(other_org)
    .bind(format!("other-{other_org}"))
    .bind(other_user)
    .execute(&app.pool)
    .await
    .expect("seed the other organization");
    let foreign = seed_category(&app, other_org).await;

    let denied = call(
        &app,
        reqwest::Method::POST,
        suppliers_url(&app, ""),
        Some(full_body(foreign)),
    )
    .await;
    assert_eq!(denied.0, StatusCode::FORBIDDEN, "no role cannot write");

    grant_reference_manage(&app).await;
    let (status, refused) = call(
        &app,
        reqwest::Method::POST,
        suppliers_url(&app, ""),
        Some(full_body(foreign)),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{refused}");

    sqlx::query("DELETE FROM expense_categories WHERE org_id = $1")
        .bind(other_org)
        .execute(&app.pool)
        .await
        .expect("clear");
    sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(other_org)
        .execute(&app.pool)
        .await
        .expect("clear");
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(other_user)
        .execute(&app.pool)
        .await
        .expect("clear");
    forget(&app).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn spec_crm_6_11_a_registration_number_of_9_or_14_digits_or_empty_is_accepted() {
    let app = harness::start().await;
    grant_reference_manage(&app).await;

    for accepted in [
        json!("732829320"),
        json!("73282932000074"),
        json!("732 829 320 00074"),
        json!(""),
        Value::Null,
    ] {
        let (status, created) = call(
            &app,
            reqwest::Method::POST,
            suppliers_url(&app, ""),
            Some(json!({ "name": "Fournisseur", "registration_number": accepted })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{accepted}: {created}");
    }
    for refused in ["12345", "1234567890", "7328293200007X", "732829320000741"] {
        let (status, body) = call(
            &app,
            reqwest::Method::POST,
            suppliers_url(&app, ""),
            Some(json!({ "name": "Fournisseur", "registration_number": refused })),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{refused}: {body}"
        );
        assert_eq!(body["details"]["field"], json!("registration_number"));
        assert_eq!(body["code"], json!("E_UNPROCESSABLE_ENTITY"));
    }

    let (status, blank) = call(
        &app,
        reqwest::Method::POST,
        suppliers_url(&app, ""),
        Some(json!({ "name": "  " })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{blank}");
    assert_eq!(blank["details"]["field"], json!("name"));

    forget(&app).await;
    app.cleanup().await;
}
