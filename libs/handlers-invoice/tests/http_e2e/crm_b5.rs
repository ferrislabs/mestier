use chrono::{Datelike, Duration, NaiveDate, Utc};
use reqwest::StatusCode;
use serde_json::{Value, json};

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

async fn draft(app: &App, lines: Value) -> Value {
    let (status, body) = post(
        app,
        app.invoices_url(),
        json!({
            "kind": "STANDARD",
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "title": "Salle de bain",
            "lines": lines,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    body["data"].clone()
}

async fn single_line_draft(app: &App, price: i32) -> Value {
    draft(
        app,
        json!([{ "label": "Pose", "quantity": "1", "unit_price_cents": price }]),
    )
    .await
}

fn id_of(invoice: &Value) -> String {
    invoice["id"].as_str().expect("an id").to_owned()
}

async fn issue(app: &App, invoice_id: &str, issued_on: Option<NaiveDate>) -> (StatusCode, Value) {
    post(
        app,
        format!("{}/issue", app.invoice_url(invoice_id)),
        json!({ "issued_on": issued_on }),
    )
    .await
}

async fn issued(app: &App, price: i32) -> Value {
    let created = single_line_draft(app, price).await;
    let (status, body) = issue(app, &id_of(&created), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    body["data"].clone()
}

async fn fetch(app: &App, invoice_id: &str) -> Value {
    let (status, body) = get(app, app.invoice_url(invoice_id)).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    body["data"].clone()
}

async fn pay(app: &App, invoice_id: &str, amount: i32) -> (StatusCode, Value) {
    post(
        app,
        format!("{}/payments", app.invoice_url(invoice_id)),
        json!({
            "amount_cents": amount,
            "paid_on": Utc::now().date_naive(),
            "method": "TRANSFER",
        }),
    )
    .await
}

async fn credit(app: &App, invoice_id: &str, body: Value) -> (StatusCode, Value) {
    post(
        app,
        format!("{}/credit-notes", app.invoice_url(invoice_id)),
        body,
    )
    .await
}

async fn from_quote(app: &App, quote_id: uuid::Uuid, body: Value) -> (StatusCode, Value) {
    post(
        app,
        format!("{}/api/v1/quotes/{quote_id}/invoices", app.base_url),
        body,
    )
    .await
}

fn code_of(body: &Value) -> &str {
    body["code"].as_str().unwrap_or_default()
}

fn field_of(body: &Value) -> &str {
    body["details"]["field"].as_str().unwrap_or_default()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_7_a_full_invoice_from_a_quote_deducts_the_issued_deposit_per_rate() {
    let app = crate::harness::start().await;
    app.make_organization_subject_to_vat().await;
    let quote_id = app
        .seed_quote_with_lines(
            "ACCEPTED",
            &[
                ("2", 50_000, Some(2000), "10"),
                ("1", 20_000, Some(1000), "0"),
            ],
        )
        .await;

    let (status, deposit) = from_quote(
        &app,
        quote_id,
        json!({ "mode": "DEPOSIT", "percent": "30" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{deposit}");
    assert_eq!(deposit["data"]["kind"], json!("DEPOSIT"));
    assert_eq!(deposit["data"]["status"], json!("DRAFT"));
    assert_eq!(deposit["data"]["quote_id"], json!(quote_id));
    let deposit_id = id_of(&deposit["data"]);

    let (_, early_full) = from_quote(&app, quote_id, json!({ "mode": "FULL" })).await;
    assert_eq!(
        early_full["data"]["lines"].as_array().map(Vec::len),
        Some(2),
        "a draft deposit is not deducted: {early_full}"
    );
    let (status, _) = send(
        &app,
        client().delete(app.invoice_url(&id_of(&early_full["data"]))),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, issued_deposit) = issue(&app, &deposit_id, None).await;
    assert_eq!(status, StatusCode::OK, "{issued_deposit}");

    let (status, full) = from_quote(&app, quote_id, json!({ "mode": "FULL" })).await;
    assert_eq!(status, StatusCode::CREATED, "{full}");
    let lines = full["data"]["lines"].as_array().expect("lines");
    assert_eq!(lines.len(), 4, "{full}");
    assert_eq!(lines[0]["discount_percent"], json!("10"));
    assert_eq!(lines[2]["unit_price_cents"], json!(-6_000));
    assert_eq!(lines[2]["vat_rate_basis_points"], json!(1000));
    assert_eq!(lines[3]["unit_price_cents"], json!(-27_000));
    assert_eq!(lines[3]["vat_rate_basis_points"], json!(2000));
    assert!(
        lines[2]["label"]
            .as_str()
            .unwrap_or_default()
            .contains(issued_deposit["data"]["number"].as_str().unwrap_or("?"))
    );
    assert_eq!(full["data"]["net_cents"], json!(90_000 + 20_000 - 33_000));
    assert_eq!(full["data"]["title"], json!("Salle de bain"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_8_a_deposit_has_one_lump_line_per_rate_and_is_refused_above_the_remaining_percent()
 {
    let app = crate::harness::start().await;
    app.make_organization_subject_to_vat().await;
    let quote_id = app
        .seed_quote_with_lines(
            "ACCEPTED",
            &[
                ("1", 100_000, Some(2000), "0"),
                ("1", 50_000, Some(1000), "0"),
            ],
        )
        .await;

    let (status, deposit) = from_quote(
        &app,
        quote_id,
        json!({ "mode": "DEPOSIT", "percent": "60" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{deposit}");
    let lines = deposit["data"]["lines"].as_array().expect("lines");
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["unit_price_cents"], json!(30_000));
    assert_eq!(lines[0]["vat_rate_basis_points"], json!(1000));
    assert_eq!(lines[1]["unit_price_cents"], json!(60_000));
    assert_eq!(lines[0]["quantity"], json!("1"));

    let (status, refused) = from_quote(
        &app,
        quote_id,
        json!({ "mode": "DEPOSIT", "percent": "40.1" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(code_of(&refused), "E_DEPOSIT_EXCEEDS_REMAINING");
    assert_eq!(field_of(&refused), "percent");

    let (status, exact) = from_quote(
        &app,
        quote_id,
        json!({ "mode": "DEPOSIT", "percent": "40" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{exact}");

    let (status, view) = get(
        &app,
        format!("{}/api/v1/quotes/{quote_id}/invoicing", app.base_url),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{view}");
    assert_eq!(view["data"]["invoiced_percent"], json!("100"));
    assert_eq!(view["data"]["can_full"], json!(true));
    assert_eq!(view["data"]["can_deposit"], json!(false));
    assert_eq!(view["data"]["blocker"], json!("DEPOSITS_COVER_QUOTE"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_9_an_active_standard_invoice_blocks_the_quote_and_a_cancelled_one_does_not() {
    let app = crate::harness::start().await;
    let quote_id = app
        .seed_quote_with_lines("ACCEPTED", &[("1", 100_000, None, "0")])
        .await;

    let (status, full) = from_quote(&app, quote_id, json!({ "mode": "FULL" })).await;
    assert_eq!(status, StatusCode::CREATED, "{full}");
    let full_id = id_of(&full["data"]);

    let (status, again) = from_quote(&app, quote_id, json!({ "mode": "FULL" })).await;
    assert_eq!(status, StatusCode::CONFLICT, "{again}");
    assert_eq!(code_of(&again), "E_QUOTE_ALREADY_INVOICED");
    let (status, deposit) = from_quote(
        &app,
        quote_id,
        json!({ "mode": "DEPOSIT", "percent": "10" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{deposit}");

    let (status, _) = issue(&app, &full_id, None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, note) = credit(
        &app,
        &full_id,
        json!({ "reason": "Annulation", "full": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{note}");

    let (status, after) = from_quote(&app, quote_id, json!({ "mode": "FULL" })).await;
    assert_eq!(status, StatusCode::CREATED, "{after}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_7_a_quote_that_is_not_accepted_cannot_be_invoiced() {
    let app = crate::harness::start().await;
    let quote_id = app
        .seed_quote_with_lines("SENT", &[("1", 100_000, None, "0")])
        .await;

    let (status, body) = from_quote(&app, quote_id, json!({ "mode": "FULL" })).await;

    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(code_of(&body), "E_CONFLICT");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_1_issuing_refuses_a_customer_without_a_billing_address_or_a_siret() {
    let app = crate::harness::start().await;
    let created = single_line_draft(&app, 10_000).await;
    let invoice_id = id_of(&created);

    app.remove_customer_address().await;
    let (status, body) = issue(&app, &invoice_id, None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(field_of(&body), "billing_address");

    sqlx::query(
        "UPDATE customer_contexts SET address_line = '1 rue A', city = 'Lyon' WHERE id = $1",
    )
    .bind(app.customer_context_id)
    .execute(&app.pool)
    .await
    .expect("restore the address");
    sqlx::query("UPDATE customers SET registration_number = NULL WHERE id = $1")
        .bind(app.customer_id)
        .execute(&app.pool)
        .await
        .expect("clear the siret");
    let (status, body) = issue(&app, &invoice_id, None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(field_of(&body), "registration_number");

    app.make_customer_an_individual().await;
    let (status, body) = issue(&app, &invoice_id, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_2_to_5_4_the_issue_date_picks_the_number_year_and_the_due_date() {
    let app = crate::harness::start().await;
    let today = Utc::now().date_naive();
    let first = single_line_draft(&app, 10_000).await;
    let second = single_line_draft(&app, 10_000).await;

    let (status, future) = issue(&app, &id_of(&first), Some(today + Duration::days(1))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{future}");
    assert_eq!(code_of(&future), "E_INVOICE_DATE_RANGE");

    let earlier = today - Duration::days(10);
    let (status, issued) = issue(&app, &id_of(&first), Some(earlier)).await;
    assert_eq!(status, StatusCode::OK, "{issued}");
    assert!(
        issued["data"]["number"]
            .as_str()
            .unwrap_or_default()
            .starts_with(&format!("FAC-{}-", earlier.year())),
        "{issued}"
    );
    assert_eq!(issued["data"]["issued_on"], json!(earlier));
    assert_eq!(
        issued["data"]["due_on"],
        json!(earlier + Duration::days(30)),
        "{issued}"
    );
    assert_eq!(issued["data"]["display_state"], json!("UNPAID"));

    let (status, before_last) =
        issue(&app, &id_of(&second), Some(earlier - Duration::days(1))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{before_last}");
    assert_eq!(code_of(&before_last), "E_INVOICE_DATE_RANGE");
    assert_eq!(fetch(&app, &id_of(&second)).await["status"], json!("DRAFT"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_5_a_draft_carries_the_penalty_and_the_recovery_fee_for_a_professional() {
    let app = crate::harness::start().await;

    let professional = single_line_draft(&app, 10_000).await;
    app.make_customer_an_individual().await;
    let individual = single_line_draft(&app, 10_000).await;

    let professional_notice = professional["legal_notice"].as_str().unwrap_or_default();
    assert!(
        professional_notice.contains("Pénalités de retard :"),
        "{professional}"
    );
    assert!(
        professional_notice.contains("Indemnité forfaitaire pour frais de recouvrement"),
        "{professional}"
    );
    let individual_notice = individual["legal_notice"].as_str().unwrap_or_default();
    assert!(individual_notice.contains("Pénalités de retard :"));
    assert!(!individual_notice.contains("Indemnité"));
    assert_eq!(professional["payment_terms_days"], json!(30));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_6_an_issued_invoice_cannot_be_edited_or_deleted_and_a_draft_can_be_deleted() {
    let app = crate::harness::start().await;
    let draft_invoice = single_line_draft(&app, 10_000).await;
    let issued_invoice = issued(&app, 10_000).await;

    let (status, _) = send(
        &app,
        client().delete(app.invoice_url(&id_of(&draft_invoice))),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, gone) = get(&app, app.invoice_url(&id_of(&draft_invoice))).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{gone}");

    let (status, locked) = send(
        &app,
        client().delete(app.invoice_url(&id_of(&issued_invoice))),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{locked}");
    assert_eq!(code_of(&locked), "E_INVOICE_LOCKED");

    let (status, edit) = send(
        &app,
        client().patch(app.invoice_url(&id_of(&issued_invoice))),
        Some(json!({
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "lines": [{ "label": "x", "quantity": "1", "unit_price_cents": 1 }]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{edit}");
    assert_eq!(code_of(&edit), "E_INVOICE_LOCKED");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_10_and_5_11_a_payment_is_bounded_by_the_balance_and_a_refund_by_the_collection()
{
    let app = crate::harness::start().await;
    let invoice = issued(&app, 10_000).await;
    let invoice_id = id_of(&invoice);

    let (status, over) = pay(&app, &invoice_id, 10_001).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{over}");
    assert_eq!(code_of(&over), "E_PAYMENT_EXCEEDS_BALANCE");
    let (status, zero) = pay(&app, &invoice_id, 0).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{zero}");
    assert_eq!(field_of(&zero), "amount_cents");
    let (status, future) = post(
        &app,
        format!("{}/payments", app.invoice_url(&invoice_id)),
        json!({
            "amount_cents": 100,
            "paid_on": Utc::now().date_naive() + Duration::days(1),
            "method": "CASH",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{future}");
    assert_eq!(field_of(&future), "paid_on");
    let (status, _) = post(
        &app,
        format!("{}/payments", app.invoice_url(&invoice_id)),
        json!({
            "amount_cents": 100,
            "paid_on": Utc::now().date_naive(),
            "method": "Virement",
        }),
    )
    .await;
    assert!(
        status.is_client_error(),
        "an unknown method is refused: {status}"
    );

    let (status, ok) = pay(&app, &invoice_id, 6_000).await;
    assert_eq!(status, StatusCode::CREATED, "{ok}");
    assert_eq!(ok["data"]["method"], json!("TRANSFER"));

    let (status, too_much) = pay(&app, &invoice_id, -6_001).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{too_much}");
    assert_eq!(code_of(&too_much), "E_REFUND_EXCEEDS_COLLECTED");
    let (status, refund) = pay(&app, &invoice_id, -2_500).await;
    assert_eq!(status, StatusCode::CREATED, "{refund}");

    let read = fetch(&app, &invoice_id).await;
    assert_eq!(read["paid_cents"], json!(3_500));
    assert_eq!(read["balance_cents"], json!(6_500));
    assert_eq!(read["display_state"], json!("PARTIAL"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_12_and_5_18_list_rows_carry_paid_credited_balance_and_display_state() {
    let app = crate::harness::start().await;
    let paid = issued(&app, 10_000).await;
    let refundable = issued(&app, 8_000).await;
    let draft_invoice = single_line_draft(&app, 700).await;
    let overdue = issued(&app, 5_000).await;
    assert_eq!(
        pay(&app, &id_of(&paid), 10_000).await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        pay(&app, &id_of(&refundable), 8_000).await.0,
        StatusCode::CREATED
    );
    let (status, note) = credit(
        &app,
        &id_of(&refundable),
        json!({ "reason": "Malfaçon", "full": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{note}");
    sqlx::query("UPDATE invoices SET due_at = now() - interval '3 days' WHERE id = $1::uuid")
        .bind(id_of(&overdue))
        .execute(&app.pool)
        .await
        .expect("age the invoice");

    let (status, listed) = get(&app, app.invoices_url()).await;
    assert_eq!(status, StatusCode::OK, "{listed}");

    let rows = listed["data"].as_array().expect("rows");
    let row = |id: &str| rows.iter().find(|row| row["id"] == json!(id)).expect("row");
    assert_eq!(row(&id_of(&paid))["display_state"], json!("PAID"));
    assert_eq!(row(&id_of(&paid))["paid_cents"], json!(10_000));
    assert_eq!(row(&id_of(&paid))["balance_cents"], json!(0));
    assert_eq!(
        row(&id_of(&refundable))["display_state"],
        json!("REFUND_DUE")
    );
    assert_eq!(row(&id_of(&refundable))["credited_cents"], json!(8_000));
    assert_eq!(row(&id_of(&refundable))["balance_cents"], json!(-8_000));
    assert_eq!(row(&id_of(&draft_invoice))["display_state"], json!("DRAFT"));
    assert_eq!(row(&id_of(&overdue))["display_state"], json!("OVERDUE"));
    assert_eq!(row(&id_of(&overdue))["balance_cents"], json!(5_000));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_14_to_5_16_credit_notes_are_bounded_by_the_source_lines() {
    let app = crate::harness::start().await;
    let created = draft(
        &app,
        json!([
            { "label": "Carrelage", "quantity": "4", "unit_price_cents": 5_000 },
            { "label": "Joints", "quantity": "1", "unit_price_cents": 30_000 }
        ]),
    )
    .await;
    let invoice_id = id_of(&created);
    let first_line = created["lines"][0]["id"].clone();
    let (status, _) = issue(&app, &invoice_id, None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, no_reason) =
        credit(&app, &invoice_id, json!({ "reason": "  ", "full": true })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{no_reason}");
    assert_eq!(field_of(&no_reason), "reason");

    let (status, over) = credit(
        &app,
        &invoice_id,
        json!({ "reason": "Malfaçon", "lines": [{ "source_line_id": first_line, "quantity": "4.5" }] }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{over}");
    assert_eq!(code_of(&over), "E_CREDIT_EXCEEDS_REMAINING");

    let (status, partial) = credit(
        &app,
        &invoice_id,
        json!({ "reason": "Malfaçon", "lines": [{ "source_line_id": first_line, "quantity": "4" }] }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{partial}");
    assert_eq!(partial["data"]["net_cents"], json!(20_000));
    assert_eq!(partial["data"]["full_cancellation"], json!(false));
    assert_eq!(partial["data"]["reason"], json!("Malfaçon"));
    assert!(
        partial["data"]["number"]
            .as_str()
            .unwrap_or_default()
            .starts_with(&format!("AV-{}-", Utc::now().year())),
        "{partial}"
    );

    let (status, rest) = credit(
        &app,
        &invoice_id,
        json!({ "reason": "Solde", "full": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{rest}");
    assert_eq!(rest["data"]["full_cancellation"], json!(true));
    assert_eq!(rest["data"]["lines"].as_array().map(Vec::len), Some(1));

    let source = fetch(&app, &invoice_id).await;
    assert_eq!(source["status"], json!("ISSUED"));
    assert_eq!(source["display_state"], json!("CANCELLED"));
    assert_eq!(source["net_cents"], json!(50_000));
    assert_eq!(source["balance_cents"], json!(0));

    let (status, nothing_left) =
        credit(&app, &invoice_id, json!({ "reason": "x", "full": true })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{nothing_left}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_14_a_draft_cannot_be_credited() {
    let app = crate::harness::start().await;
    let created = single_line_draft(&app, 10_000).await;

    let (status, body) = credit(
        &app,
        &id_of(&created),
        json!({ "reason": "x", "full": true }),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(code_of(&body), "E_CONFLICT");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_5_17_sending_reminding_and_noting_record_events_without_moving_the_balance() {
    let app = crate::harness::start().await;
    let invoice = issued(&app, 10_000).await;
    let invoice_id = id_of(&invoice);
    sqlx::query("UPDATE invoices SET due_at = now() - interval '6 days' WHERE id = $1::uuid")
        .bind(&invoice_id)
        .execute(&app.pool)
        .await
        .expect("age the invoice");

    let (status, sent) = post(
        &app,
        format!("{}/send", app.invoice_url(&invoice_id)),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{sent}");
    assert!(sent["data"]["sent_at"].is_string());

    let (status, reminded) = post(
        &app,
        format!("{}/remind", app.invoice_url(&invoice_id)),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{reminded}");

    let (status, noted) = post(
        &app,
        format!("{}/note", app.invoice_url(&invoice_id)),
        json!({ "note": "  Rappeler mardi  " }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{noted}");
    assert_eq!(noted["data"]["internal_note"], json!("Rappeler mardi"));

    let read = fetch(&app, &invoice_id).await;
    let events = read["events"].as_array().expect("events");
    assert_eq!(
        events
            .iter()
            .map(|event| event["kind"].clone())
            .collect::<Vec<_>>(),
        vec![json!("SENT"), json!("REMINDER"), json!("NOTE")]
    );
    assert!(
        events[1]["detail"]
            .as_str()
            .unwrap_or_default()
            .starts_with("Relance envoyée ("),
        "{read}"
    );
    assert_eq!(events[2]["detail"], json!("Rappeler mardi"));
    assert_eq!(read["balance_cents"], json!(10_000));

    let created = single_line_draft(&app, 10_000).await;
    let (status, refused) = post(
        &app,
        format!("{}/send", app.invoice_url(&id_of(&created))),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");

    app.cleanup().await;
}
