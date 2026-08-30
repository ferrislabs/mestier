//! End-to-end tests for the invoice API (#319).
//!
//! Same shape as `handlers-quote`'s own suite: a request enters over a real
//! socket and crosses the whole stack — typed routing, rate-limit and auth
//! middleware with a real RS256 token checked against a real JWKS fetch,
//! the handler, the use case, a Postgres transaction, the event write that
//! transaction commits. Nothing in that chain is doubled, which is also why
//! these are `#[ignore]`d.
//!
//! ```bash
//! docker compose up -d postgres redis
//! source .env
//! cargo test -p handlers-invoice --test http_e2e -- --ignored
//! ```

mod harness;
mod issuer;

use serde_json::{Value, json};

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn create_draft(app: &harness::App, project_id: Option<uuid::Uuid>) -> Value {
    client()
        .post(app.invoices_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "kind": "STANDARD",
            "project_id": project_id,
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "due_at": null,
            "notes": null,
            "operation_nature": null,
            "delivery_address": null,
            "lines": [
                { "label": "Pose de faience", "quantity": "4", "unit_price_cents": 4500 },
                { "label": "Depose ancien carrelage", "quantity": "12.5", "unit_price_cents": 3800 }
            ]
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the create answer is json")
}

async fn create_single_line_draft(app: &harness::App, unit_price_cents: i32) -> Value {
    client()
        .post(app.invoices_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "kind": "STANDARD",
            "project_id": null,
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "due_at": null,
            "notes": null,
            "operation_nature": null,
            "delivery_address": null,
            "lines": [
                { "label": "Solde travaux", "quantity": "1", "unit_price_cents": unit_price_cents }
            ]
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the create answer is json")
}

async fn issue(app: &harness::App, invoice_id: &str) -> Value {
    client()
        .post(format!("{}/issue", app.invoice_url(invoice_id)))
        .bearer_auth(&app.token)
        .json(&json!({ "allow_exceeding_total": false }))
        .send()
        .await
        .expect("the api answers the issue call")
        .json()
        .await
        .expect("the issue answer is json")
}

/// #395: a member with no role at all — bare membership, the gate this
/// issue closes — is refused reading the organization's invoices, not
/// served a silent empty list.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_view_invoices_is_refused_the_list() {
    let app = harness::start().await;

    let response = client()
        .get(app.invoices_url())
        .bearer_auth(&app.no_role_token)
        .send()
        .await
        .expect("the api answers the no-role member's list call");

    assert_eq!(
        response.status(),
        403,
        "VIEW_INVOICES must gate the list, not just organization membership"
    );

    app.cleanup().await;
}

/// #395: the same no-role member is refused creating an invoice —
/// `MANAGE_INVOICES` gates the write side the way `VIEW_INVOICES` gates
/// reads above.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_manage_invoices_is_refused_creating_an_invoice() {
    let app = harness::start().await;

    let response = client()
        .post(app.invoices_url())
        .bearer_auth(&app.no_role_token)
        .json(&json!({
            "kind": "STANDARD",
            "project_id": null,
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "due_at": null,
            "notes": null,
            "operation_nature": null,
            "delivery_address": null,
            "lines": [
                { "label": "Pose de faience", "quantity": "4", "unit_price_cents": 4500 }
            ]
        }))
        .send()
        .await
        .expect("the api answers the no-role member's create call");

    assert_eq!(
        response.status(),
        403,
        "MANAGE_INVOICES must gate creation, not just organization membership"
    );

    app.cleanup().await;
}

/// An unauthenticated call is turned away before it ever reaches a handler
/// — the same first assertion `handlers-quote`'s suite opens with, proving
/// the auth middleware is really in this crate's own layer chain and not
/// just inherited on paper.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_unauthenticated_call_is_turned_away() {
    let app = harness::start().await;

    let response = client()
        .post(app.invoices_url())
        .json(&json!({
            "kind": "STANDARD",
            "project_id": null,
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "due_at": null,
            "notes": null,
            "operation_nature": null,
            "delivery_address": null,
            "lines": []
        }))
        .send()
        .await
        .expect("the api answers an unauthenticated call");

    assert_eq!(
        response.status(),
        401,
        "the auth middleware must be in the chain"
    );

    app.cleanup().await;
}

/// The whole draft lifecycle over HTTP: created, priced by the server,
/// listed, read back, and edited — with the server recomputing the total
/// every time, never trusting a client-sent figure.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_draft_invoice_is_priced_persisted_listed_and_patched() {
    let app = harness::start().await;

    let created = create_draft(&app, None).await;
    let body = &created["data"];
    // 4 x 4500 + 12.5 x 3800 = 18 000 + 47 500 = 65 500, and the seeded
    // organization is not subject to VAT, so gross equals net.
    assert_eq!(body["net_cents"], json!(65_500), "{created}");
    assert_eq!(body["gross_cents"], json!(65_500), "{created}");
    assert_eq!(body["vat_breakdown"], json!([]), "{created}");
    assert_eq!(body["status"], json!("DRAFT"), "{created}");
    assert_eq!(body["kind"], json!("STANDARD"), "{created}");
    assert_eq!(body["number"], json!(null), "{created}");
    let invoice_id = body["id"].as_str().expect("an id").to_owned();

    let listed: Value = client()
        .get(app.invoices_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");
    let found = listed["data"]
        .as_array()
        .expect("the list answer carries an array")
        .iter()
        .any(|invoice| invoice["id"] == json!(invoice_id));
    assert!(found, "the invoice just created is missing from {listed}");

    let fetched: Value = client()
        .get(app.invoice_url(&invoice_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call")
        .json()
        .await
        .expect("the get answer is json");
    assert_eq!(fetched["data"]["id"], json!(invoice_id), "{fetched}");

    let patched: Value = client()
        .patch(app.invoice_url(&invoice_id))
        .bearer_auth(&app.token)
        .json(&json!({
            "project_id": null,
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "due_at": null,
            "notes": "Revu a la baisse",
            "operation_nature": null,
            "delivery_address": null,
            "lines": [
                { "label": "Pose de faience", "quantity": "3", "unit_price_cents": 4500 }
            ]
        }))
        .send()
        .await
        .expect("the api answers the patch call")
        .json()
        .await
        .expect("the patch answer is json");
    assert_eq!(patched["data"]["net_cents"], json!(13_500), "{patched}");
    assert_eq!(
        patched["data"]["notes"],
        json!("Revu a la baisse"),
        "{patched}"
    );

    app.cleanup().await;
}

/// Issuing allocates the number and locks the document from further edits
/// — proving the type-level immutability `DraftInvoice`/`Invoice` enforce
/// in the domain actually reaches HTTP, not just the domain's own unit
/// tests.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn issuing_an_invoice_allocates_a_number_and_locks_it_from_further_edits() {
    let app = harness::start().await;

    let created = create_single_line_draft(&app, 50_000).await;
    let invoice_id = created["data"]["id"].as_str().expect("an id").to_owned();

    let issued = issue(&app, &invoice_id).await;
    let number = issued["data"]["number"]
        .as_str()
        .expect("a number once issued")
        .to_owned();
    assert!(number.starts_with("FAC-"), "{number}");
    assert_eq!(issued["data"]["status"], json!("ISSUED"), "{issued}");

    let refused = client()
        .patch(app.invoice_url(&invoice_id))
        .bearer_auth(&app.token)
        .json(&json!({
            "project_id": null,
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "due_at": null,
            "notes": null,
            "operation_nature": null,
            "delivery_address": null,
            "lines": [
                { "label": "Solde travaux", "quantity": "1", "unit_price_cents": 60_000 }
            ]
        }))
        .send()
        .await
        .expect("the api answers the second patch call");
    assert_eq!(
        refused.status(),
        409,
        "an issued invoice must refuse a further edit"
    );

    app.cleanup().await;
}

/// A deposit and a final invoice, issued straight from a project's quote —
/// `issue_deposit`/`issue_final_invoice` (#317), reachable over HTTP for
/// the first time by this issue. Also covers the project billing summary
/// (#319's own new read): after both acts, nothing is left to bill.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_deposit_and_a_final_invoice_bill_a_project_against_its_quote() {
    let app = harness::start().await;

    let quote_id = app.seed_quote(100_000).await;
    let project_id = app.seed_project(Some(quote_id)).await;

    let deposit: Value = client()
        .post(format!(
            "{}/api/v1/projects/{project_id}/invoices/deposit",
            app.base_url
        ))
        .bearer_auth(&app.token)
        .json(&json!({
            "percentage_bp": 3000,
            "due_at": null,
            "notes": null,
            "allow_exceeding_total": false
        }))
        .send()
        .await
        .expect("the api answers the deposit call")
        .json()
        .await
        .expect("the deposit answer is json");
    assert_eq!(deposit["data"]["kind"], json!("DEPOSIT"), "{deposit}");
    assert_eq!(deposit["data"]["status"], json!("ISSUED"), "{deposit}");
    assert_eq!(deposit["data"]["net_cents"], json!(30_000), "{deposit}");

    let final_invoice: Value = client()
        .post(format!(
            "{}/api/v1/projects/{project_id}/invoices/final",
            app.base_url
        ))
        .bearer_auth(&app.token)
        .json(&json!({ "due_at": null, "notes": null, "allow_exceeding_total": false }))
        .send()
        .await
        .expect("the api answers the final invoice call")
        .json()
        .await
        .expect("the final invoice answer is json");
    assert_eq!(
        final_invoice["data"]["kind"],
        json!("FINAL"),
        "{final_invoice}"
    );
    assert_eq!(
        final_invoice["data"]["status"],
        json!("ISSUED"),
        "{final_invoice}"
    );
    // The remainder of a 100 000 quote after a 30 000 deposit.
    assert_eq!(
        final_invoice["data"]["net_cents"],
        json!(70_000),
        "{final_invoice}"
    );

    let summary: Value = client()
        .get(format!(
            "{}/api/v1/projects/{project_id}/billing-summary",
            app.base_url
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the billing summary call")
        .json()
        .await
        .expect("the billing summary answer is json");
    assert_eq!(summary["data"]["quoted_cents"], json!(100_000), "{summary}");
    assert_eq!(summary["data"]["billed_cents"], json!(100_000), "{summary}");
    assert_eq!(summary["data"]["remaining_cents"], json!(0), "{summary}");

    let listed: Value = client()
        .get(format!(
            "{}/api/v1/projects/{project_id}/invoices",
            app.base_url
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the project invoice listing call")
        .json()
        .await
        .expect("the project invoice listing answer is json");
    assert_eq!(
        listed["data"].as_array().expect("an array").len(),
        2,
        "{listed}"
    );

    app.cleanup().await;
}

/// Recording a payment never writes a status column — it changes what the
/// next read derives. This is the one place that design (#320) becomes
/// observable end-to-end: the same invoice reads `ISSUED`, then
/// `PARTIALLY_PAID`, then `PAID`, with nothing but payments recorded in
/// between.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn recording_payments_changes_the_status_the_next_read_derives() {
    let app = harness::start().await;

    let created = create_single_line_draft(&app, 20_000).await;
    let invoice_id = created["data"]["id"].as_str().expect("an id").to_owned();
    let issued = issue(&app, &invoice_id).await;
    assert_eq!(issued["data"]["status"], json!("ISSUED"), "{issued}");

    let paid_on = harness::now_storable().date_naive();

    let after_partial = client()
        .post(format!("{}/payments", app.invoice_url(&invoice_id)))
        .bearer_auth(&app.token)
        .json(&json!({
            "amount_cents": 10_000,
            "paid_on": paid_on,
            "method": "bank_transfer",
            "reference": null,
            "note": null,
            "allow_exceeding_total": false
        }))
        .send()
        .await
        .expect("the api answers the record-payment call");
    assert_eq!(
        after_partial.status(),
        201,
        "{}",
        after_partial.text().await.unwrap_or_default()
    );

    let refetched_partial: Value = client()
        .get(app.invoice_url(&invoice_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call")
        .json()
        .await
        .expect("the get answer is json");
    assert_eq!(
        refetched_partial["data"]["status"],
        json!("PARTIALLY_PAID"),
        "{refetched_partial}"
    );

    let second_payment: Value = client()
        .post(format!("{}/payments", app.invoice_url(&invoice_id)))
        .bearer_auth(&app.token)
        .json(&json!({
            "amount_cents": 10_000,
            "paid_on": paid_on,
            "method": "bank_transfer",
            "reference": null,
            "note": null,
            "allow_exceeding_total": false
        }))
        .send()
        .await
        .expect("the api answers the second record-payment call")
        .json()
        .await
        .expect("the second record-payment answer is json");

    let refetched_paid: Value = client()
        .get(app.invoice_url(&invoice_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call")
        .json()
        .await
        .expect("the get answer is json");
    assert_eq!(
        refetched_paid["data"]["status"],
        json!("PAID"),
        "{refetched_paid}"
    );

    let payments_list: Value = client()
        .get(format!("{}/payments", app.invoice_url(&invoice_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list-payments call")
        .json()
        .await
        .expect("the list-payments answer is json");
    assert_eq!(
        payments_list["data"].as_array().expect("an array").len(),
        2,
        "{payments_list}"
    );

    // The delete route is reached by payment id alone (no invoice_id in the
    // path), which is exactly what `require_payment_membership` and the
    // `find_payment_by_id` use case this issue adds exist to make correct.
    let second_payment_id = second_payment["data"]["id"]
        .as_str()
        .expect("a payment id")
        .to_owned();
    let deleted = client()
        .delete(format!(
            "{}/api/v1/invoice-payments/{second_payment_id}",
            app.base_url
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete-payment call");
    assert_eq!(
        deleted.status(),
        204,
        "{}",
        deleted.text().await.unwrap_or_default()
    );

    let refetched_after_delete: Value = client()
        .get(app.invoice_url(&invoice_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call")
        .json()
        .await
        .expect("the get answer is json");
    assert_eq!(
        refetched_after_delete["data"]["status"],
        json!("PARTIALLY_PAID"),
        "deleting the second payment must be reflected on the next read: {refetched_after_delete}"
    );

    app.cleanup().await;
}

/// A credit note is the only way to correct an issued invoice (#318) — this
/// proves it is reachable over HTTP, `source_invoice_id` comes from the
/// path rather than the body, and the credit note is priced like any other
/// document.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_credit_note_corrects_an_issued_invoice_over_http() {
    let app = harness::start().await;

    let created = create_single_line_draft(&app, 30_000).await;
    let invoice_id = created["data"]["id"].as_str().expect("an id").to_owned();
    issue(&app, &invoice_id).await;

    let credit_note: Value = client()
        .post(format!("{}/credit-notes", app.invoice_url(&invoice_id)))
        .bearer_auth(&app.token)
        .json(&json!({
            "lines": [
                { "label": "Remise geste commercial", "quantity": "1", "unit_price_cents": 10_000 }
            ],
            "notes": "Geste commercial",
            "allow_exceeding_invoice_total": false
        }))
        .send()
        .await
        .expect("the api answers the credit-note call")
        .json()
        .await
        .expect("the credit-note answer is json");

    assert_eq!(
        credit_note["data"]["kind"],
        json!("CREDIT_NOTE"),
        "{credit_note}"
    );
    assert_eq!(
        credit_note["data"]["net_cents"],
        json!(10_000),
        "{credit_note}"
    );
    assert_eq!(
        credit_note["data"]["source_invoice_id"],
        json!(invoice_id),
        "{credit_note}"
    );
    assert_eq!(
        credit_note["data"]["status"],
        json!("ISSUED"),
        "{credit_note}"
    );

    app.cleanup().await;
}

/// The seeded organization carries no legal identity here — that is the
/// point of this test, via `start_with_incomplete_identity`. #310 makes
/// completeness a type-level fact; this proves the refusal actually
/// reaches an HTTP caller as a 409 naming what is missing, not as a blank
/// PDF or a 500. Every other test in this suite uses the complete-identity
/// default so this refusal does not block them.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn pdf_export_refuses_an_incomplete_legal_identity_naming_the_missing_fields() {
    let app = harness::start_with_incomplete_identity().await;
    let created = create_single_line_draft(&app, 10_000).await;
    let invoice_id = created["data"]["id"].as_str().expect("an id").to_owned();

    let response = client()
        .get(format!("{}/pdf", app.invoice_url(&invoice_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the pdf call");

    assert_eq!(
        response.status(),
        409,
        "an incomplete identity is a 409, not a 200 or a 500"
    );
    let body: Value = response.json().await.expect("a json error body");
    let message = body["message"].as_str().unwrap_or_default();
    for field in [
        "legal_name",
        "legal_form",
        "registration_number",
        "insurance_mention",
        "vat_status",
        "address_line1",
        "address_postal_code",
        "address_city",
        "address_country",
    ] {
        assert!(
            message.contains(field),
            "the refusal must name `{field}` as missing: {message}"
        );
    }

    app.cleanup().await;
}

/// The other side of the same rule: the default harness seeds a complete
/// legal identity, so the export actually produces a PDF, and the content
/// stream carries the mentions #310/#341 exist specifically to print.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn pdf_export_renders_once_the_legal_identity_is_complete() {
    let app = harness::start().await;
    let created = create_single_line_draft(&app, 10_000).await;
    let invoice_id = created["data"]["id"].as_str().expect("an id").to_owned();

    let response = client()
        .get(format!("{}/pdf", app.invoice_url(&invoice_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the pdf call");

    assert_eq!(
        response.status(),
        200,
        "a complete identity must be allowed to export"
    );
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("application/pdf")
    );
    let bytes = response.bytes().await.expect("the pdf body");
    assert!(
        bytes.starts_with(b"%PDF-1.4"),
        "the response must be an actual PDF, not a placeholder"
    );
    let text = String::from_utf8_lossy(&bytes);
    for expected in [
        "Acme SARL",
        "SARL",
        "123 456 789 00012",
        "Article 293 B du CGI",
    ] {
        assert!(
            text.contains(expected),
            "the pdf content stream must carry `{expected}`"
        );
    }

    app.cleanup().await;
}

/// #342's `?format=facturx` reaches the handler and is refused with a
/// precise 409 — not a 200, not a 500 — because the harness's seeded
/// customer carries no SIREN. This is the real, current state of the
/// feature end to end: the organization side of `ElectronicInvoicingFacts`
/// is satisfiable (the harness seeds a complete legal identity), the
/// customer side is not, and the invoice must be issued first regardless.
/// See `infrastructure::invoice::facturx::cii`'s own module doc comment in
/// `mestier-core` for the sibling gap this does not reach in this
/// particular fixture (the buyer's postal address): here the request is
/// refused one check earlier, on the SIREN, before the EN 16931 validator
/// is ever invoked.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn facturx_export_refuses_a_customer_with_no_siren() {
    let app = harness::start().await;
    let created = create_single_line_draft(&app, 10_000).await;
    let invoice_id = created["data"]["id"].as_str().expect("an id").to_owned();
    issue(&app, &invoice_id).await;

    let response = client()
        .get(format!(
            "{}/pdf?format=facturx",
            app.invoice_url(&invoice_id)
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the pdf call");

    assert_eq!(
        response.status(),
        409,
        "a customer with no SIREN must be a named refusal, not a 200, a 422 or a 500"
    );
    let body: Value = response.json().await.expect("a json error body");
    let message = body["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("customer_registration_number"),
        "the refusal must name the missing SIREN: {message}"
    );

    app.cleanup().await;
}

/// The plain `?format=pdf` (and the bare default) must still work exactly
/// as before #342 touched this route — the format parameter is additive,
/// never a behaviour change for a caller that never asks for it.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn pdf_export_default_format_is_unchanged_by_the_new_query_parameter() {
    let app = harness::start().await;
    let created = create_single_line_draft(&app, 10_000).await;
    let invoice_id = created["data"]["id"].as_str().expect("an id").to_owned();

    let explicit = client()
        .get(format!("{}/pdf?format=pdf", app.invoice_url(&invoice_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the pdf call");
    assert_eq!(explicit.status(), 200);

    let bare = client()
        .get(format!("{}/pdf", app.invoice_url(&invoice_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the pdf call");
    assert_eq!(bare.status(), 200);

    app.cleanup().await;
}
