use chrono::{Datelike, Duration, NaiveDate, Utc};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::App;

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn send(
    app: &App,
    request: reqwest::RequestBuilder,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request = request.bearer_auth(&app.token);
    let request = match body {
        Some(body) => request.json(&body),
        None => request,
    };
    let response = request.send().await.expect("the api answers");
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    let body = serde_json::from_str(&text).unwrap_or(Value::Null);

    (status, body)
}

async fn post(app: &App, url: String, body: Value) -> (StatusCode, Value) {
    send(app, client().post(url), Some(body)).await
}

async fn get(app: &App, url: String) -> (StatusCode, Value) {
    send(app, client().get(url), None).await
}

fn org_url(app: &App, tail: &str) -> String {
    format!(
        "{}/api/v1/organizations/{}/{tail}",
        app.base_url, app.organization_id
    )
}

fn expense_url(app: &App, id: &str) -> String {
    format!("{}/api/v1/expenses/{id}", app.base_url)
}

fn code_of(body: &Value) -> &str {
    body["code"].as_str().unwrap_or_default()
}

fn field_of(body: &Value) -> &str {
    body["details"]["field"].as_str().unwrap_or_default()
}

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

async fn categories(app: &App) -> Vec<Value> {
    let (status, body) = get(app, org_url(app, "expense-categories")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["data"].as_array().expect("categories").clone()
}

async fn first_category(app: &App) -> String {
    categories(app).await[0]["id"]
        .as_str()
        .expect("a category id")
        .to_owned()
}

fn expense_body(category_id: &str, date: NaiveDate) -> Value {
    json!({
        "date": date,
        "label": "Quincaillerie et visserie",
        "category_id": category_id,
        "amount_ht_cents": 84_200,
        "vat_cents": 16_840,
        "due_date": date + Duration::days(20),
        "reference": "FA-88213",
    })
}

async fn create_expense(app: &App, body: Value) -> (StatusCode, Value) {
    post(app, org_url(app, "expenses"), body).await
}

async fn member_id(app: &App) -> Uuid {
    sqlx::query_scalar("SELECT id FROM organization_members WHERE organization_id = $1 AND user_id IS NOT NULL ORDER BY created_at LIMIT 1")
        .bind(app.organization_id)
        .fetch_one(&app.pool)
        .await
        .expect("a member")
}

async fn issued_invoice(app: &App, unit_price: i32, vat_bp: Option<i32>) -> Value {
    let (status, created) = post(
        app,
        app.invoices_url(),
        json!({
            "kind": "STANDARD",
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "title": "Salle de bain",
            "lines": [{
                "label": "Pose",
                "quantity": "1",
                "unit_price_cents": unit_price,
                "vat_rate_basis_points": vat_bp,
            }],
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["data"]["id"].as_str().expect("an id").to_owned();
    let (status, issued) = post(
        app,
        format!("{}/issue", app.invoice_url(&id)),
        json!({ "issued_on": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{issued}");

    issued["data"].clone()
}

async fn ledger(app: &App, query: &str) -> Value {
    let (status, body) = get(app, org_url(app, &format!("ledger{query}"))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["data"].clone()
}

async fn foreign_organization(app: &App) -> Uuid {
    let user_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("other-{user_id}@example.com"))
    .bind(format!("other-{user_id}"))
    .bind("Other")
    .bind(format!("sub-other-{user_id}"))
    .execute(&app.pool)
    .await
    .expect("seed the other user");
    let org_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, 'Other', $2, $3)",
    )
    .bind(org_id)
    .bind(format!("other-{org_id}"))
    .bind(user_id)
    .execute(&app.pool)
    .await
    .expect("seed the other organization");

    org_id
}

async fn forget_foreign(app: &App, org_id: Uuid) {
    for table in ["expenses", "unallocated_payments", "expense_categories"] {
        let _ = sqlx::query(&format!("DELETE FROM {table} WHERE org_id = $1"))
            .bind(org_id)
            .execute(&app.pool)
            .await;
    }
    let _ = sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(org_id)
        .execute(&app.pool)
        .await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_1_an_expense_needs_a_label_an_amount_a_vat_a_category_and_a_due_date_not_before_its_date()
 {
    let app = crate::harness::start().await;
    let category = first_category(&app).await;
    let date = today();

    let cases = [
        (json!({ "label": "  " }), "label"),
        (json!({ "amount_ht_cents": 0 }), "amount_ht_cents"),
        (json!({ "amount_ht_cents": -5 }), "amount_ht_cents"),
        (json!({ "vat_cents": -1 }), "vat_cents"),
        (json!({ "due_date": date - Duration::days(1) }), "due_date"),
    ];
    for (patch, field) in cases {
        let mut body = expense_body(&category, date);
        for (key, value) in patch.as_object().expect("a patch") {
            body[key] = value.clone();
        }
        let (status, refused) = create_expense(&app, body).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{field}: {refused}"
        );
        assert_eq!(field_of(&refused), field);
        assert_eq!(code_of(&refused), "E_UNPROCESSABLE_ENTITY");
    }

    let (status, unknown) =
        create_expense(&app, expense_body(&Uuid::now_v7().to_string(), date)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{unknown}");

    let other = foreign_organization(&app).await;
    let foreign_category = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO expense_categories (id, org_id, name, hue) VALUES ($1, $2, 'Autres', 0)",
    )
    .bind(foreign_category)
    .bind(other)
    .execute(&app.pool)
    .await
    .expect("seed a foreign category");
    let (status, foreign) =
        create_expense(&app, expense_body(&foreign_category.to_string(), date)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{foreign}");

    let mut free_vat = expense_body(&category, date);
    free_vat["vat_cents"] = json!(0);
    free_vat["due_date"] = json!(date);
    let (status, created) = create_expense(&app, free_vat).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["data"]["status"], json!("TO_PAY"));
    assert_eq!(created["data"]["total_cents"], json!(84_200));
    assert_eq!(created["data"]["display_state"], json!("TO_PAY"));

    let (status, listed) = get(&app, org_url(&app, "expenses")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["data"].as_array().map(Vec::len), Some(1));

    forget_foreign(&app, other).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_2_an_advanced_expense_is_created_paid_by_card_without_due_date() {
    let app = crate::harness::start().await;
    let category = first_category(&app).await;
    let member = member_id(&app).await;
    let date = today() - Duration::days(3);
    let mut body = expense_body(&category, date);
    body["advanced_by_member_id"] = json!(member);

    let (status, created) = create_expense(&app, body).await;

    assert_eq!(status, StatusCode::CREATED, "{created}");
    let data = &created["data"];
    assert_eq!(data["status"], json!("PAID"));
    assert_eq!(data["paid_on"], json!(date));
    assert_eq!(data["method"], json!("CARD"));
    assert_eq!(data["due_date"], Value::Null);
    assert_eq!(data["reimbursed_on"], Value::Null);
    assert_eq!(data["display_state"], json!("TO_REIMBURSE"));

    let mut stranger = expense_body(&category, date);
    stranger["advanced_by_member_id"] = json!(Uuid::now_v7());
    let (status, refused) = create_expense(&app, stranger).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{refused}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_3_marking_paid_works_once_on_an_expense_to_pay_that_nobody_advanced() {
    let app = crate::harness::start().await;
    let category = first_category(&app).await;
    let member = member_id(&app).await;
    let (_, created) = create_expense(&app, expense_body(&category, today())).await;
    let id = created["data"]["id"].as_str().expect("an id").to_owned();
    let pay_url = format!("{}/pay", expense_url(&app, &id));
    let paid_on = today();

    let (status, paid) = post(
        &app,
        pay_url.clone(),
        json!({ "paid_on": paid_on, "method": "TRANSFER" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{paid}");
    assert_eq!(paid["data"]["status"], json!("PAID"));
    assert_eq!(paid["data"]["paid_on"], json!(paid_on));
    assert_eq!(paid["data"]["method"], json!("TRANSFER"));

    let (status, again) = post(
        &app,
        pay_url,
        json!({ "paid_on": paid_on - Duration::days(1), "method": "CASH" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{again}");
    assert_eq!(code_of(&again), "E_EXPENSE_NOT_PAYABLE");
    let (_, unchanged) = get(&app, expense_url(&app, &id)).await;
    assert_eq!(unchanged["data"]["method"], json!("TRANSFER"));

    let mut advanced = expense_body(&category, today());
    advanced["advanced_by_member_id"] = json!(member);
    let (_, advanced) = create_expense(&app, advanced).await;
    let advanced_id = advanced["data"]["id"].as_str().expect("an id").to_owned();
    let (status, refused) = post(
        &app,
        format!("{}/pay", expense_url(&app, &advanced_id)),
        json!({ "paid_on": paid_on, "method": "CASH" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_4_reimbursing_works_once_on_an_advanced_expense() {
    let app = crate::harness::start().await;
    let category = first_category(&app).await;
    let member = member_id(&app).await;
    let mut advanced = expense_body(&category, today() - Duration::days(2));
    advanced["advanced_by_member_id"] = json!(member);
    let (_, created) = create_expense(&app, advanced).await;
    let id = created["data"]["id"].as_str().expect("an id").to_owned();
    let url = format!("{}/reimburse", expense_url(&app, &id));

    let (status, first) = post(&app, url.clone(), json!({})).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["data"]["reimbursed_on"], json!(today()));
    assert_eq!(first["data"]["display_state"], json!("REIMBURSED"));

    let (status, second) = post(&app, url, json!({})).await;
    assert_eq!(status, StatusCode::CONFLICT, "{second}");
    assert_eq!(code_of(&second), "E_EXPENSE_NOT_REIMBURSABLE");

    let (_, plain) = create_expense(&app, expense_body(&category, today())).await;
    let plain_id = plain["data"]["id"].as_str().expect("an id").to_owned();
    let (status, refused) = post(
        &app,
        format!("{}/reimburse", expense_url(&app, &plain_id)),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_expense_is_edited_and_deleted_at_any_status_and_a_foreign_one_is_refused() {
    let app = crate::harness::start().await;
    let category = first_category(&app).await;
    let (_, created) = create_expense(&app, expense_body(&category, today())).await;
    let id = created["data"]["id"].as_str().expect("an id").to_owned();
    let (_, _) = post(
        &app,
        format!("{}/pay", expense_url(&app, &id)),
        json!({ "paid_on": today(), "method": "CARD" }),
    )
    .await;

    let mut edited = expense_body(&category, today());
    edited["label"] = json!("Carburant");
    edited["amount_ht_cents"] = json!(9_600);
    edited["proof_file_key"] = json!("proofs/ticket.jpg");
    let (status, updated) = send(&app, client().patch(expense_url(&app, &id)), Some(edited)).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["data"]["label"], json!("Carburant"));
    assert_eq!(updated["data"]["status"], json!("PAID"));
    assert_eq!(
        updated["data"]["proof_file_key"],
        json!("proofs/ticket.jpg")
    );

    let (status, _) = send(&app, client().delete(expense_url(&app, &id)), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = get(&app, expense_url(&app, &id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let other = foreign_organization(&app).await;
    let foreign_category = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO expense_categories (id, org_id, name, hue) VALUES ($1, $2, 'Autres', 0)",
    )
    .bind(foreign_category)
    .bind(other)
    .execute(&app.pool)
    .await
    .expect("seed a foreign category");
    let foreign_expense = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO expenses (id, org_id, date, label, category_id, amount_ht_cents) \
         VALUES ($1, $2, CURRENT_DATE, 'Etranger', $3, 100)",
    )
    .bind(foreign_expense)
    .bind(other)
    .bind(foreign_category)
    .execute(&app.pool)
    .await
    .expect("seed a foreign expense");
    let (status, refused) = get(&app, expense_url(&app, &foreign_expense.to_string())).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{refused}");
    let (status, _) = send(
        &app,
        client().delete(expense_url(&app, &foreign_expense.to_string())),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let still_there: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM expenses WHERE id = $1")
        .bind(foreign_expense)
        .fetch_one(&app.pool)
        .await
        .expect("count");
    assert_eq!(still_there, 1);

    forget_foreign(&app, other).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_categories_are_seeded_once_and_a_used_one_cannot_be_deleted() {
    let app = crate::harness::start().await;

    let first = categories(&app).await;
    let second = categories(&app).await;
    assert_eq!(first.len(), 9);
    assert_eq!(second.len(), 9);

    let (status, created) = post(
        &app,
        org_url(&app, "expense-categories"),
        json!({ "name": "Formation", "hue": 300 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let new_id = created["data"]["id"].as_str().expect("an id").to_owned();
    let (status, duplicate) = post(
        &app,
        org_url(&app, "expense-categories"),
        json!({ "name": "formation", "hue": 10 }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{duplicate}");
    assert_eq!(field_of(&duplicate), "name");
    let (status, bad_hue) = post(
        &app,
        org_url(&app, "expense-categories"),
        json!({ "name": "Autre chose", "hue": 400 }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{bad_hue}");
    assert_eq!(field_of(&bad_hue), "hue");

    let category_url = format!("{}/api/v1/expense-categories/{new_id}", app.base_url);
    let (status, renamed) = send(
        &app,
        client().patch(&category_url),
        Some(json!({ "name": "Formations", "hue": 301 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{renamed}");
    assert_eq!(renamed["data"]["name"], json!("Formations"));

    create_expense(&app, expense_body(&new_id, today())).await;
    let (status, in_use) = send(&app, client().delete(&category_url), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{in_use}");
    assert_eq!(code_of(&in_use), "E_EXPENSE_CATEGORY_IN_USE");

    let free = first[1]["id"].as_str().expect("an id");
    let (status, _) = send(
        &app,
        client().delete(format!("{}/api/v1/expense-categories/{free}", app.base_url)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_5_to_6_7_the_journal_signs_summarizes_and_trends() {
    let app = crate::harness::start().await;
    let category = first_category(&app).await;
    let member = member_id(&app).await;
    let invoice = issued_invoice(&app, 100_000, None).await;
    let invoice_id = invoice["id"].as_str().expect("an id").to_owned();
    let invoice_number = invoice["number"].as_str().expect("a number").to_owned();
    let day = today();

    let (status, _) = post(
        &app,
        format!("{}/payments", app.invoice_url(&invoice_id)),
        json!({ "amount_cents": 60_000, "paid_on": day, "method": "TRANSFER" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = post(
        &app,
        format!("{}/payments", app.invoice_url(&invoice_id)),
        json!({ "amount_cents": -10_000, "paid_on": day, "method": "TRANSFER" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, receipt) = post(
        &app,
        org_url(&app, "receipts"),
        json!({ "amount_cents": 4_000, "paid_on": day, "method": "CHECK", "note": "Acompte Fabre", "customer_id": app.customer_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{receipt}");

    create_expense(&app, expense_body(&category, day)).await;
    let mut late = expense_body(&category, day - Duration::days(30));
    late["due_date"] = json!(day - Duration::days(3));
    late["label"] = json!("Assurance");
    late["amount_ht_cents"] = json!(2_000);
    late["vat_cents"] = json!(400);
    create_expense(&app, late).await;
    let mut advanced = expense_body(&category, day);
    advanced["advanced_by_member_id"] = json!(member);
    advanced["label"] = json!("Petit materiel");
    advanced["amount_ht_cents"] = json!(1_000);
    advanced["vat_cents"] = json!(200);
    create_expense(&app, advanced).await;

    let data = ledger(&app, "").await;

    let entries = data["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 6);
    let dates: Vec<&str> = entries
        .iter()
        .map(|entry| entry["date"].as_str().expect("a date"))
        .collect();
    let mut sorted = dates.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(dates, sorted, "newest first");

    let amount_of = |label: &str| -> i64 {
        entries
            .iter()
            .find(|entry| entry["label"] == json!(label))
            .map(|entry| entry["amount_cents"].as_i64().expect("an amount"))
            .expect("an entry")
    };
    assert_eq!(amount_of("Quincaillerie et visserie"), -101_040);
    let receipts: Vec<&Value> = entries
        .iter()
        .filter(|e| e["kind"] == json!("RECEIPT"))
        .collect();
    let mut receipt_amounts: Vec<i64> = receipts
        .iter()
        .map(|entry| entry["amount_cents"].as_i64().expect("an amount"))
        .collect();
    receipt_amounts.sort();
    assert_eq!(receipt_amounts, vec![-10_000, 4_000, 60_000]);
    let paid = receipts
        .iter()
        .find(|entry| entry["amount_cents"] == json!(60_000))
        .expect("the payment");
    assert_eq!(paid["invoice_number"], json!(invoice_number));
    assert_eq!(paid["status"], json!("DONE"));
    let unallocated = receipts
        .iter()
        .find(|entry| entry["amount_cents"] == json!(4_000))
        .expect("the receipt");
    assert_eq!(unallocated["status"], json!("UNALLOCATED"));
    assert_eq!(unallocated["label"], json!("Acompte Fabre"));

    let summary = &data["summary"];
    assert_eq!(summary["receipts_cents"], json!(54_000));
    assert_eq!(summary["expenses_cents"], json!(101_040 + 2_400 + 1_200));
    assert_eq!(summary["net_cents"], json!(54_000 - 104_640));
    assert_eq!(summary["pending_cents"], json!(101_040 + 2_400));
    assert_eq!(summary["unallocated_cents"], json!(4_000));
    assert_eq!(data["payables"]["total_cents"], json!(101_040 + 2_400));
    assert_eq!(data["payables"]["overdue_cents"], json!(2_400));

    let trend = data["trend"].as_array().expect("trend");
    assert_eq!(trend.len(), 6);
    let this_month = format!("{:04}-{:02}", day.year(), day.month());
    assert_eq!(trend[5]["month"], json!(this_month));
    assert_eq!(trend[5]["receipts_cents"], json!(54_000));
    let empty = trend
        .iter()
        .filter(|month| month["receipts_cents"] == json!(0) && month["expenses_cents"] == json!(0))
        .count();
    assert!(empty >= 3, "empty months stay in the trend: {trend:?}");

    let filtered = ledger(&app, &format!("?month={this_month}")).await;
    assert_eq!(filtered["summary"]["receipts_cents"], json!(54_000));
    let (status, refused) = get(&app, org_url(&app, "ledger?month=2026-13")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(field_of(&refused), "month");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_8_vat_collected_is_invoices_minus_credit_notes_and_due_is_the_difference() {
    let app = crate::harness::start().await;
    app.make_organization_subject_to_vat().await;
    let category = first_category(&app).await;
    let invoice = issued_invoice(&app, 100_000, Some(2000)).await;
    let invoice_id = invoice["id"].as_str().expect("an id").to_owned();
    assert_eq!(invoice["gross_cents"], json!(120_000));

    let (status, note) = post(
        &app,
        format!("{}/credit-notes", app.invoice_url(&invoice_id)),
        json!({ "reason": "Geste commercial", "full": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{note}");
    let credit_vat = note["data"]["gross_cents"].as_i64().unwrap_or_default()
        - note["data"]["net_cents"].as_i64().unwrap_or_default();
    assert_eq!(credit_vat, 20_000);
    let second = issued_invoice(&app, 50_000, Some(1000)).await;
    assert_eq!(second["gross_cents"], json!(55_000));

    let mut fuel = expense_body(&category, today());
    fuel["amount_ht_cents"] = json!(10_000);
    fuel["vat_cents"] = json!(2_000);
    create_expense(&app, fuel).await;

    let data = ledger(&app, "").await;

    assert_eq!(data["vat"]["collected_cents"], json!(5_000));
    assert_eq!(data["vat"]["deductible_cents"], json!(2_000));
    assert_eq!(data["vat"]["due_cents"], json!(3_000));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_9_an_unallocated_receipt_is_allocated_to_an_issued_invoice_within_its_balance()
{
    let app = crate::harness::start().await;
    let invoice = issued_invoice(&app, 100_000, None).await;
    let invoice_id = invoice["id"].as_str().expect("an id").to_owned();

    let (status, zero) = post(
        &app,
        org_url(&app, "receipts"),
        json!({ "amount_cents": 0, "paid_on": today(), "method": "CHECK" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{zero}");
    assert_eq!(field_of(&zero), "amount_cents");
    let (status, future) = post(
        &app,
        org_url(&app, "receipts"),
        json!({ "amount_cents": 100, "paid_on": today() + Duration::days(2), "method": "CHECK" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{future}");
    assert_eq!(field_of(&future), "paid_on");
    let (status, unknown_customer) = post(
        &app,
        org_url(&app, "receipts"),
        json!({ "amount_cents": 100, "paid_on": today(), "method": "CHECK", "customer_id": Uuid::now_v7() }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{unknown_customer}");

    let (_, too_much) = post(
        &app,
        org_url(&app, "receipts"),
        json!({ "amount_cents": 150_000, "paid_on": today(), "method": "TRANSFER" }),
    )
    .await;
    let too_much_id = too_much["data"]["id"].as_str().expect("an id").to_owned();
    let (status, exceeded) = post(
        &app,
        format!("{}/api/v1/receipts/{too_much_id}/allocate", app.base_url),
        json!({ "invoice_id": invoice_id }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{exceeded}");
    assert_eq!(code_of(&exceeded), "E_PAYMENT_EXCEEDS_BALANCE");
    let still_unallocated = ledger(&app, "").await;
    assert_eq!(
        still_unallocated["summary"]["unallocated_cents"],
        json!(150_000)
    );

    let (_, fitting) = post(
        &app,
        org_url(&app, "receipts"),
        json!({ "amount_cents": 40_000, "paid_on": today(), "method": "CHECK", "reference": "CHQ 42" }),
    )
    .await;
    let fitting_id = fitting["data"]["id"].as_str().expect("an id").to_owned();
    let (status, allocated) = post(
        &app,
        format!("{}/api/v1/receipts/{fitting_id}/allocate", app.base_url),
        json!({ "invoice_id": invoice_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{allocated}");
    assert_eq!(allocated["data"]["amount_cents"], json!(40_000));
    assert_eq!(allocated["data"]["method"], json!("CHECK"));

    let (status, balance) = get(&app, format!("{}/balance", app.invoice_url(&invoice_id))).await;
    assert_eq!(status, StatusCode::OK, "{balance}");
    let after = ledger(&app, "").await;
    assert_eq!(after["summary"]["unallocated_cents"], json!(150_000));
    let entries = after["entries"].as_array().expect("entries");
    assert!(
        entries
            .iter()
            .any(|entry| entry["amount_cents"] == json!(40_000)
                && entry["invoice_id"] == json!(invoice_id))
    );

    let (status, gone) = post(
        &app,
        format!("{}/api/v1/receipts/{fitting_id}/allocate", app.base_url),
        json!({ "invoice_id": invoice_id }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{gone}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_treasury_needs_the_invoice_permissions() {
    let app = crate::harness::start().await;

    for url in [
        org_url(&app, "ledger"),
        org_url(&app, "expenses"),
        org_url(&app, "expense-categories"),
    ] {
        let response = client()
            .get(url)
            .bearer_auth(&app.no_role_token)
            .send()
            .await
            .expect("the api answers");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    let response = client()
        .post(org_url(&app, "expenses"))
        .bearer_auth(&app.no_role_token)
        .json(&json!({
            "date": today(), "label": "X", "category_id": Uuid::now_v7(), "amount_ht_cents": 100
        }))
        .send()
        .await
        .expect("the api answers");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    app.cleanup().await;
}
