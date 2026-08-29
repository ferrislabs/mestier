//! End-to-end tests for the purchasing API (#339).
//!
//! Same shape as `handlers-invoice`'s own suite: a request enters over a
//! real socket and crosses the whole stack — typed routing, rate-limit and
//! auth middleware with a real RS256 token checked against a real JWKS
//! fetch, the handler, the use case, a Postgres transaction, the event
//! write that transaction commits, and — for `import` — a real object
//! storage upload. Nothing in that chain is doubled, which is also why
//! these are `#[ignore]`d.
//!
//! ```bash
//! docker compose up -d postgres redis
//! source .env
//! cargo test -p handlers-purchase --test http_e2e -- --ignored
//! ```

mod harness;
mod issuer;

use serde_json::{Value, json};

/// A real, valid Factur-X PDF (#337's own fixture): number `F20260023`,
/// supplier `LE FOURNISSEUR`, 3 lines.
const VALID_PDF: &[u8] =
    include_bytes!("../../../core/src/infrastructure/supplier_invoice/facturx/fixtures/valid.pdf");

/// A PDF with no readable Factur-X attachment — #337's `AttachmentExtraction`
/// failure mode, kept here rather than silently discarded.
const MALFORMED_PDF: &[u8] = include_bytes!(
    "../../../core/src/infrastructure/supplier_invoice/facturx/fixtures/malformed.pdf"
);

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn import(app: &harness::App, bytes: &'static [u8]) -> Value {
    client()
        .post(app.import_url())
        .bearer_auth(&app.token)
        .header("content-type", "application/pdf")
        .body(bytes)
        .send()
        .await
        .expect("the api answers the import call")
        .json()
        .await
        .expect("the import answer is json")
}

/// An unauthenticated call is turned away before it ever reaches a handler
/// — the same first assertion every e2e suite in this repository opens
/// with, proving the auth middleware is really in this crate's own layer
/// chain and not just inherited on paper.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_unauthenticated_call_is_turned_away() {
    let app = harness::start().await;

    let response = client()
        .post(app.import_url())
        .header("content-type", "application/pdf")
        .body(VALID_PDF)
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

/// The whole import path: the file is both stored and parsed (#339: "the
/// file is stored, not only parsed"), the invoice lands `Received`, and a
/// review note can be attached without touching any of the document's own
/// fields.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn importing_a_valid_file_stores_it_and_creates_a_received_invoice() {
    let app = harness::start().await;

    let imported = import(&app, VALID_PDF).await;
    let body = &imported["data"];
    assert_eq!(body["outcome"], json!("created"), "{imported}");
    let invoice = &body["invoice"];
    assert_eq!(invoice["number"], json!("F20260023"), "{imported}");
    assert_eq!(
        invoice["supplier_name"],
        json!("LE FOURNISSEUR"),
        "{imported}"
    );
    assert_eq!(invoice["status"], json!("RECEIVED"), "{imported}");
    assert_eq!(
        invoice["lines"].as_array().expect("lines").len(),
        3,
        "{imported}"
    );
    assert!(
        invoice["source_file_key"].as_str().is_some(),
        "the original file must have been stored: {imported}"
    );
    let invoice_id = invoice["id"].as_str().expect("an id").to_owned();

    let listed: Value = client()
        .get(app.supplier_invoices_url())
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
        .any(|item| item["id"] == json!(invoice_id));
    assert!(found, "the invoice just imported is missing from {listed}");

    let patched: Value = client()
        .patch(app.supplier_invoice_url(&invoice_id))
        .bearer_auth(&app.token)
        .json(&json!({ "notes": "A verifier avec le comptable" }))
        .send()
        .await
        .expect("the api answers the patch call")
        .json()
        .await
        .expect("the patch answer is json");
    assert_eq!(
        patched["data"]["notes"],
        json!("A verifier avec le comptable"),
        "{patched}"
    );
    // The document's own fields must survive an untouched PATCH.
    assert_eq!(patched["data"]["number"], json!("F20260023"), "{patched}");

    app.cleanup().await;
}

/// #337's most important rule reachable over HTTP: the same document
/// imported twice is refused as a conflict, never silently duplicated.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn importing_the_same_invoice_twice_is_refused() {
    let app = harness::start().await;

    let first = import(&app, VALID_PDF).await;
    assert_eq!(first["data"]["outcome"], json!("created"), "{first}");

    let second = client()
        .post(app.import_url())
        .bearer_auth(&app.token)
        .header("content-type", "application/pdf")
        .body(VALID_PDF)
        .send()
        .await
        .expect("the api answers the second import call");
    assert_eq!(
        second.status(),
        409,
        "the same document imported twice must be refused, not duplicated"
    );

    app.cleanup().await;
}

/// A file with no readable Factur-X attachment is kept, with the reason —
/// #337's binding rule, surfaced over HTTP as a 200 whose body says which
/// of the two outcomes actually happened rather than a generic failure.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_file_that_cannot_be_parsed_is_kept_with_its_reason() {
    let app = harness::start().await;

    let response = client()
        .post(app.import_url())
        .bearer_auth(&app.token)
        .header("content-type", "application/pdf")
        .body(MALFORMED_PDF)
        .send()
        .await
        .expect("the api answers the import call");
    assert_eq!(response.status(), 200, "a parse failure is not a 4xx/5xx");

    let body: Value = response.json().await.expect("a json body");
    assert_eq!(body["data"]["outcome"], json!("parse_failed"), "{body}");
    assert!(
        body["data"]["reason"]
            .as_str()
            .is_some_and(|r| !r.is_empty()),
        "the reason must be stated: {body}"
    );

    app.cleanup().await;
}

/// Confirming and rejecting are the two ends of the review — reachable over
/// HTTP, each with its own reviewer note.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn confirming_and_rejecting_move_a_supplier_invoice_through_its_review() {
    let app = harness::start().await;

    let confirmed_source = import(&app, VALID_PDF).await;
    let confirmed_id = confirmed_source["data"]["invoice"]["id"]
        .as_str()
        .expect("an id")
        .to_owned();

    let confirmed: Value = client()
        .post(format!(
            "{}/confirm",
            app.supplier_invoice_url(&confirmed_id)
        ))
        .bearer_auth(&app.token)
        .json(&json!({ "notes": "Montant verifie" }))
        .send()
        .await
        .expect("the api answers the confirm call")
        .json()
        .await
        .expect("the confirm answer is json");
    assert_eq!(
        confirmed["data"]["status"],
        json!("CONFIRMED"),
        "{confirmed}"
    );
    assert_eq!(
        confirmed["data"]["notes"],
        json!("Montant verifie"),
        "{confirmed}"
    );

    let refused = client()
        .post(format!(
            "{}/reject",
            app.supplier_invoice_url(&confirmed_id)
        ))
        .bearer_auth(&app.token)
        .json(&json!({ "notes": null }))
        .send()
        .await
        .expect("the api answers the second review call");
    assert_eq!(
        refused.status(),
        409,
        "an already-reviewed invoice must refuse a second review"
    );

    app.cleanup().await;
}

/// #339's full-replace `PUT` against a project's cost — allocating,
/// re-allocating with a changed amount, and reading the project's own
/// running total back.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn replacing_line_allocations_is_reflected_in_the_project_s_supplier_costs() {
    let app = harness::start().await;
    let project_id = app.seed_project().await;

    let imported = import(&app, VALID_PDF).await;
    let invoice = &imported["data"]["invoice"];
    let line_id = invoice["lines"][0]["id"]
        .as_str()
        .expect("a line id")
        .to_owned();
    let line_total_cents = invoice["lines"][0]["line_total_cents"]
        .as_i64()
        .expect("a line total") as i32;

    let replaced: Value = client()
        .put(app.line_allocations_url(&line_id))
        .bearer_auth(&app.token)
        .json(&json!({
            "allocations": [
                { "project_id": project_id, "amount_cents": line_total_cents }
            ]
        }))
        .send()
        .await
        .expect("the api answers the replace call")
        .json()
        .await
        .expect("the replace answer is json");
    let allocations = replaced["data"].as_array().expect("an array");
    assert_eq!(allocations.len(), 1, "{replaced}");
    assert_eq!(
        allocations[0]["amount_cents"],
        json!(line_total_cents),
        "{replaced}"
    );

    let costs: Value = client()
        .get(app.project_supplier_costs_url(&project_id.to_string()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the project costs call")
        .json()
        .await
        .expect("the project costs answer is json");
    assert_eq!(
        costs["data"]["allocated_cents"],
        json!(line_total_cents),
        "{costs}"
    );
    // #340's own requirement: each cost line links back to the invoice it
    // came from, not just a bare total.
    let cost_lines = costs["data"]["lines"].as_array().expect("an array");
    assert_eq!(cost_lines.len(), 1, "{costs}");
    assert_eq!(
        cost_lines[0]["supplier_invoice_number"],
        json!("F20260023"),
        "{costs}"
    );
    assert_eq!(
        cost_lines[0]["amount_cents"],
        json!(line_total_cents),
        "{costs}"
    );

    // Full-replace with an empty list must clear the allocation and the
    // project's cost along with it.
    let cleared: Value = client()
        .put(app.line_allocations_url(&line_id))
        .bearer_auth(&app.token)
        .json(&json!({ "allocations": [] }))
        .send()
        .await
        .expect("the api answers the clearing call")
        .json()
        .await
        .expect("the clearing answer is json");
    assert_eq!(
        cleared["data"].as_array().expect("an array").len(),
        0,
        "{cleared}"
    );

    let costs_after_clear: Value = client()
        .get(app.project_supplier_costs_url(&project_id.to_string()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the project costs call")
        .json()
        .await
        .expect("the project costs answer is json");
    assert_eq!(
        costs_after_clear["data"]["allocated_cents"],
        json!(0),
        "{costs_after_clear}"
    );
    assert_eq!(
        costs_after_clear["data"]["lines"]
            .as_array()
            .expect("an array")
            .len(),
        0,
        "{costs_after_clear}"
    );

    app.cleanup().await;
}
